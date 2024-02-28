use locales::t;
use modals::Modals;
use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::client::WebPkiServerVerifier;
use rustls::crypto::{verify_tls12_signature, verify_tls13_signature, WebPkiSupportedAlgorithms};
use rustls::pki_types::{CertificateDer, Der, ServerName, TrustAnchor, UnixTime};
use rustls::{CertificateError, DigitallySignedStruct, Error, RootCertStore, SignatureScheme};
use std::sync::Arc;
use webpki::ring as webpki_algs;
use xous_names::XousNames;

/// The entire purpose of the StifledCertificateVerification is to gain access to
/// the certificate-chain-of-trust offered by a host by stifling CertificateError::UnknownIssuer
///
#[derive(Debug)]
pub struct StifledCertificateVerification {
    pub roots: RootCertStore,
    pub supported: WebPkiSupportedAlgorithms,
}

impl StifledCertificateVerification {

    /// rustls includes a sanity check early in the process, to ensure that the RootCertStore
    /// contains at least one root certificate before going to the trouble of asking the host
    /// to offer its certificate chain of trust. Trouble is, xous/tls wants to probe the host
    /// to see what is offered up before deciding if we trust it.
    ///
    /// The somewhat hacky work-around is to add a single bogus TrustAnchor to RootCertStore,
    /// and neither the bogus TrustAnchor or the RootCertStore exist beyond the lifetime of
    /// this StifledCertificateVerification.
    ///
    pub fn new() -> Self {
        let mut root_cert_store = rustls::RootCertStore::empty();
        // rustls::ServerCertVerifierBuilder::build() returns a
        // `CertVerifierBuilderError` if no trust anchors have been provided.
        let single_bogus_ta_to_avoid_error_on_empty_roots = TrustAnchor {
            subject: Der::from_slice(b"bogus subject"),
            subject_public_key_info: Der::from_slice(b"bogus subject_public_key_info"),
            name_constraints: None,
        };
        root_cert_store.roots.push(single_bogus_ta_to_avoid_error_on_empty_roots);
        Self { roots: root_cert_store, supported: SUPPORTED_SIG_ALGS }
    }
}

impl ServerCertVerifier for StifledCertificateVerification {
    /// Will verify the certificate with the default rustls WebPkiVerifier,
    /// BUT specifically overrides a `CertificateError::UnknownIssuer` and
    /// return ServerCertVerified::assertion()
    fn verify_server_cert(
        &self,
        end_entity: &CertificateDer,
        intermediates: &[CertificateDer],
        server_name: &ServerName,
        ocsp: &[u8],
        now: UnixTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        match WebPkiServerVerifier::builder(Arc::new(self.roots.clone())).build() {
            Ok(rustls_default_verifier) => {
                match rustls_default_verifier.verify_server_cert(
                    end_entity,
                    intermediates,
                    server_name,
                    ocsp,
                    now,
                ) {
                    Ok(ok) => Ok(ok),
                    Err(Error::InvalidCertificate(e)) => {
                        let xns = XousNames::new().unwrap();
                        let modals = Modals::new(&xns).unwrap();
                        match e {
                            CertificateError::UnknownIssuer => Ok(ServerCertVerified::assertion()),
                            CertificateError::NotValidYet => {
                                modals
                                    .show_notification(
                                        t!("tls.probe_help_not_valid_yet", locales::LANG),
                                        None,
                                    )
                                    .expect("modal failed");
                                Err(Error::InvalidCertificate(e))
                            }
                            _ => {
                                modals
                                    .show_notification(
                                        format!(
                                            "{}\n{:?}",
                                            t!("tls.probe_invalid_certificate", locales::LANG),
                                            e
                                        )
                                        .as_str(),
                                        None,
                                    )
                                    .expect("modal failed");

                                Err(Error::InvalidCertificate(e))
                            }
                        }
                    }
                    Err(e) => Err(e),
                }
            }
            Err(e) => {
                log::warn!("failed to build WebPkiServerVerifier: {e}");
                Err(Error::General("failed to build WebPkiServerVerifier".to_string()))
            }
        }
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, Error> {
        verify_tls12_signature(message, cert, dss, &self.supported)
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, Error> {
        verify_tls13_signature(message, cert, dss, &self.supported)
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.supported.supported_schemes()
    }
}

/// Vendor in the sadly private rustls::crypto::ring::SUPPORTED_SIG_ALGS
/// A `WebPkiSupportedAlgorithms` value that reflects webpki's capabilities when
/// compiled against *ring*.
///
static SUPPORTED_SIG_ALGS: WebPkiSupportedAlgorithms = WebPkiSupportedAlgorithms {
    all: &[
        webpki_algs::ECDSA_P256_SHA256,
        webpki_algs::ECDSA_P256_SHA384,
        webpki_algs::ECDSA_P384_SHA256,
        webpki_algs::ECDSA_P384_SHA384,
        webpki_algs::ED25519,
        webpki_algs::RSA_PSS_2048_8192_SHA256_LEGACY_KEY,
        webpki_algs::RSA_PSS_2048_8192_SHA384_LEGACY_KEY,
        webpki_algs::RSA_PSS_2048_8192_SHA512_LEGACY_KEY,
        webpki_algs::RSA_PKCS1_2048_8192_SHA256,
        webpki_algs::RSA_PKCS1_2048_8192_SHA384,
        webpki_algs::RSA_PKCS1_2048_8192_SHA512,
        webpki_algs::RSA_PKCS1_3072_8192_SHA384,
    ],
    mapping: &[
        // Note: for TLS1.2 the curve is not fixed by SignatureScheme. For TLS1.3 it is.
        (
            SignatureScheme::ECDSA_NISTP384_SHA384,
            &[webpki_algs::ECDSA_P384_SHA384, webpki_algs::ECDSA_P256_SHA384],
        ),
        (
            SignatureScheme::ECDSA_NISTP256_SHA256,
            &[webpki_algs::ECDSA_P256_SHA256, webpki_algs::ECDSA_P384_SHA256],
        ),
        (SignatureScheme::ED25519, &[webpki_algs::ED25519]),
        (SignatureScheme::RSA_PSS_SHA512, &[webpki_algs::RSA_PSS_2048_8192_SHA512_LEGACY_KEY]),
        (SignatureScheme::RSA_PSS_SHA384, &[webpki_algs::RSA_PSS_2048_8192_SHA384_LEGACY_KEY]),
        (SignatureScheme::RSA_PSS_SHA256, &[webpki_algs::RSA_PSS_2048_8192_SHA256_LEGACY_KEY]),
        (SignatureScheme::RSA_PKCS1_SHA512, &[webpki_algs::RSA_PKCS1_2048_8192_SHA512]),
        (SignatureScheme::RSA_PKCS1_SHA384, &[webpki_algs::RSA_PKCS1_2048_8192_SHA384]),
        (SignatureScheme::RSA_PKCS1_SHA256, &[webpki_algs::RSA_PKCS1_2048_8192_SHA256]),
    ],
};
