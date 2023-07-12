use super::*;
//use crate::{ChatOp, Dialogue, Event, Post, CHAT_SERVER_NAME};
use core::fmt::Write;
use dialogue::{author::Author, post::Post, Dialogue};
use gam::UxRegistration;
use graphics_server::api::GlyphStyle;
use graphics_server::{DrawStyle, Gid, PixelColor, Point, Rectangle, TextBounds, TextView};
use locales::t;
use modals::Modals;
use std::io::{Error, ErrorKind};
use xous::{MessageEnvelope, CID};

use xous_names::XousNames;

#[allow(dead_code)]
pub(crate) struct Ui {
    // optional structures that indicate new input to the Chat loop per iteration
    // an input string
    pub input: Option<xous_ipc::String<{ POST_TEXT_MAX }>>,
    // messages from other servers
    msg: Option<MessageEnvelope>,

    // Pddb connection
    pddb: pddb::Pddb,
    pddb_dict: Option<String>,
    pddb_key: Option<String>,
    dialogue: Option<Dialogue>,

    // Callbacks:
    // optional SID of the "Owner" Chat App to receive UI-events
    app_cid: Option<CID>,
    // optional opcode ID to process UI-event msgs
    opcode_event: Option<usize>,

    canvas: Gid,
    gam: gam::Gam,

    // variables that define our graphical attributes
    screensize: Point,
    bubble_width: u16,
    margin: Point,        // margin to edge of canvas
    bubble_margin: Point, // margin of text in bubbles
    bubble_radius: u16,
    bubble_space: i16, // spacing between text bubbles

    // our security token for making changes to our record on the GAM
    token: [u32; 4],
}

#[allow(dead_code)]
impl Ui {
    pub(crate) fn new(
        sid: xous::SID,
        app_cid: Option<xous::CID>,
        opcode_event: Option<usize>,
    ) -> Self {
        let xns = XousNames::new().unwrap();
        let gam = gam::Gam::new(&xns).expect("can't connect to GAM");

        let token = gam
            .register_ux(UxRegistration {
                app_name: xous_ipc::String::<128>::from_str(SERVER_NAME_CHAT),
                ux_type: gam::UxType::Chat,
                predictor: Some(xous_ipc::String::<64>::from_str(
                    ime_plugin_shell::SERVER_NAME_IME_PLUGIN_SHELL,
                )),
                listener: sid.to_array(), // note disclosure of our SID to the GAM -- the secret is now shared with the GAM!
                redraw_id: ChatOp::GamRedraw as u32,
                gotinput_id: Some(ChatOp::GamLine as u32),
                audioframe_id: None,
                rawkeys_id: Some(ChatOp::GamRawKeys as u32),
                focuschange_id: Some(ChatOp::GamChangeFocus as u32),
            })
            .expect("couldn't register Ux context for chat");

        let canvas = gam
            .request_content_canvas(token.unwrap())
            .expect("couldn't get content canvas");
        let screensize = gam
            .get_canvas_bounds(canvas)
            .expect("couldn't get dimensions of content canvas");
        Ui {
            input: None,
            msg: None,
            pddb: pddb::Pddb::new(),
            pddb_dict: None,
            pddb_key: None,
            dialogue: None,
            app_cid,
            opcode_event,
            canvas,
            gam,
            screensize,
            bubble_width: ((screensize.x / 5) * 4) as u16, // 80% width for the text bubbles
            margin: Point::new(4, 4),
            bubble_margin: Point::new(4, 4),
            bubble_radius: 4,
            bubble_space: 4,
            token: token.unwrap(),
        }
    }

    // set the current Dialogue
    pub fn dialogue_set(&mut self, pddb_dict: &str, pddb_key: &str) {
        self.pddb_dict = Some(pddb_dict.to_string());
        self.pddb_key = Some(pddb_key.to_string());
    }

    // set listener sid and opcodes to receive UI-event msgs & user posts
    // pub fn listen_set(
    //     &self,
    //     cid: xous::CID,
    //     opcode_post: Option<u32>,
    //     opcode_event: Option<u32>,
    //     opcode_rawkeys: Option<u32>,
    // ) {
    //     self.listener = cid;
    //     self.opcode_post = opcode_post;
    //     self.opcode_event = opcode_event;
    //     self.opcode_rawkeys = opcode_rawkeys;
    // }

    // add a new Post to the current Dialogue
    pub fn post_add(
        &mut self,
        author: &str,
        timestamp: u32,
        text: &str,
        attach_url: Option<&str>,
    ) -> Result<(), Error> {
        match &mut self.dialogue {
            Some(ref mut dialogue) => dialogue
                .post_add(author, timestamp, text, attach_url)
                .unwrap(),
            None => log::warn!("no Dialogue available to add Post"),
        }
        Ok(())
    }

    // delete a Post from the current Dialogue
    pub fn post_del(&self, _key: u32) -> Result<(), Error> {
        log::warn!("not implemented");
        Err(Error::new(ErrorKind::Other, "not implemented"))
    }

    // get a Post from the current Dialogue
    pub fn post_find(&self, author: &str, timestamp: u32) -> Option<usize> {
        match &self.dialogue {
            Some(dialogue) => dialogue.post_find(author, timestamp),
            None => None,
        }
    }

    // get a Post from the current Dialogue
    pub fn post_get(&self, _key: u32) -> Result<Post, Error> {
        log::warn!("not implemented");
        Err(Error::new(ErrorKind::Other, "not implemented"))
    }

    // set various status flags on a Post in the current Dialogue
    pub fn post_flag(&self, _key: u32) -> Result<(), Error> {
        log::warn!("not implemented");
        Err(Error::new(ErrorKind::Other, "not implemented"))
    }

    // set the text displayed on each of the Precursor Fn buttons
    pub fn ui_button(
        &self,
        _f1: Option<&str>,
        _f2: Option<&str>,
        _f3: Option<&str>,
        _f4: Option<&str>,
    ) -> Result<(), Error> {
        log::warn!("not implemented");
        Err(Error::new(ErrorKind::Other, "not implemented"))
    }

    // request the Chat object to display a menu with options to the user
    pub fn ui_menu(&self, _options: Vec<&str>) -> Result<Vec<u32>, Error> {
        log::warn!("not implemented");
        Err(Error::new(ErrorKind::Other, "not implemented"))
    }

    fn app_cid(&self) -> Option<CID> {
        self.app_cid
    }

    // send a xous scalar message with an Event to the Chat App cid/opcode
    fn event(&self, _event: Event) {
        log::warn!("not implemented");
        // Err(Error::new(ErrorKind::Other, "not implemented"))
    }

    fn bubble(&self, post: &Post, author: &Author, baseline: i16) -> TextView {
        let mut bubble_tv = if author.flag_is(AuthorFlag::Right) {
            TextView::new(
                self.canvas,
                TextBounds::GrowableFromBr(
                    Point::new(self.screensize.x - self.margin.x, baseline),
                    self.bubble_width,
                ),
            )
        } else {
            TextView::new(
                self.canvas,
                TextBounds::GrowableFromBl(Point::new(self.margin.x, baseline), self.bubble_width),
            )
        };
        bubble_tv.border_width = 1;
        bubble_tv.draw_border = true;
        bubble_tv.clear_area = true;
        bubble_tv.rounded_border = Some(self.bubble_radius);
        bubble_tv.style = GlyphStyle::Regular;
        bubble_tv.margin = self.bubble_margin;
        bubble_tv.ellipsis = false;
        bubble_tv.insertion = None;
        write!(bubble_tv.text, "{}", post.text()).expect("couldn't write history text to TextView");
        bubble_tv
    }

    fn clear_area(&self) {
        self.gam
            .draw_rectangle(
                self.canvas,
                Rectangle::new_with_style(
                    Point::new(0, 0),
                    self.screensize,
                    DrawStyle {
                        fill_color: Some(PixelColor::Light),
                        stroke_color: None,
                        stroke_width: 0,
                    },
                ),
            )
            .expect("can't clear canvas area");
    }

    pub(crate) fn redraw(&mut self) -> Result<(), xous::Error> {
        self.clear_area();

        // this defines the bottom border of the text bubbles as they stack up wards
        let mut bubble_baseline = self.screensize.y - self.margin.y;

        if let Some(dialogue) = &self.dialogue {
            for post in dialogue.posts().rev() {
                if let Some(author) = dialogue.author(post.author_id()) {
                    let mut bubble_tv = self.bubble(post, author, bubble_baseline);
                    self.gam
                        .post_textview(&mut bubble_tv)
                        .expect("couldn't render bubble textview");

                    if let Some(bounds) = bubble_tv.bounds_computed {
                        // we only subtract 1x of the margin because the bounds were computed from a "bottom right" that already counted
                        // the margin once.
                        bubble_baseline -=
                            (bounds.br.y - bounds.tl.y) + self.bubble_space + self.bubble_margin.y;
                        if bubble_baseline <= 0 {
                            // don't draw history that overflows the top of the screen
                            break;
                        }
                    } else {
                        break; // we get None on the bounds computed if the text view fell off the top of the screen
                    }
                } else {
                    log::warn!(
                        "Post missing Author: {:?}:{:?} {:?}",
                        self.pddb_dict,
                        self.pddb_key,
                        post
                    );
                }
            }
        }
        log::trace!("chat app redraw##");
        self.gam.redraw().expect("couldn't redraw screen");
        Ok(())
    }
}
