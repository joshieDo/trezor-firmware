use super::{
    super::{
        component::{Frame, FrameMsg},
        theme,
    },
    ConfirmActionMenu, ConfirmActionStrings,
};
use crate::{
    error::Error,
    maybe_trace::MaybeTrace,
    micropython::obj::Obj,
    strutil::TString,
    ui::{
        component::{
            base::ComponentExt,
            swipe_detect::SwipeSettings,
            text::{
                paragraphs::{Paragraph, ParagraphSource, ParagraphVecShort, VecExt},
                TextStyle,
            },
            Component,
        },
        flow::{FlowMsg, Swipable, SwipeFlow, SwipePage},
        geometry::Direction,
        layout::util::{ConfirmBlob, StrOrBytes},
        model_mercury::{component::SwipeContent, flow},
    },
};
use heapless::Vec;

pub struct ConfirmBlobParams {
    title: TString<'static>,
    subtitle: Option<TString<'static>>,
    footer_instruction: Option<TString<'static>>,
    footer_description: Option<TString<'static>>,
    data: Obj,
    description: Option<TString<'static>>,
    description_font: &'static TextStyle,
    extra: Option<TString<'static>>,
    verb: Option<TString<'static>>,
    verb_cancel: Option<TString<'static>>,
    verb_info: Option<TString<'static>>,
    info_button: bool,
    cancel_button: bool,
    menu_button: bool,
    prompt: bool,
    hold: bool,
    chunkify: bool,
    text_mono: bool,
    page_limit: Option<usize>,
    swipe_up: bool,
    swipe_down: bool,
    swipe_right: bool,
}

impl ConfirmBlobParams {
    pub const fn new(
        title: TString<'static>,
        data: Obj,
        description: Option<TString<'static>>,
    ) -> Self {
        Self {
            title,
            subtitle: None,
            footer_instruction: None,
            footer_description: None,
            data,
            description,
            description_font: &theme::TEXT_NORMAL,
            extra: None,
            verb: None,
            verb_cancel: None,
            verb_info: None,
            info_button: false,
            cancel_button: false,
            menu_button: false,
            prompt: false,
            hold: false,
            chunkify: false,
            text_mono: true,
            page_limit: None,
            swipe_up: false,
            swipe_down: false,
            swipe_right: false,
        }
    }

    pub const fn with_extra(mut self, extra: Option<TString<'static>>) -> Self {
        self.extra = extra;
        self
    }

    pub const fn with_subtitle(mut self, subtitle: Option<TString<'static>>) -> Self {
        self.subtitle = subtitle;
        self
    }

    pub const fn with_menu_button(mut self) -> Self {
        self.menu_button = true;
        self
    }

    pub const fn with_cancel_button(mut self) -> Self {
        self.cancel_button = true;
        self
    }

    pub const fn with_info_button(mut self, info_button: bool) -> Self {
        self.info_button = info_button;
        self
    }

    pub const fn with_verb(mut self, verb: Option<TString<'static>>) -> Self {
        self.verb = verb;
        self
    }

    pub const fn with_verb_cancel(mut self, verb_cancel: Option<TString<'static>>) -> Self {
        self.verb_cancel = verb_cancel;
        self
    }

    pub const fn with_verb_info(mut self, verb_info: Option<TString<'static>>) -> Self {
        self.verb_info = verb_info;
        self
    }

    pub const fn with_prompt(mut self, prompt: bool) -> Self {
        self.prompt = prompt;
        self
    }

    pub const fn with_hold(mut self, hold: bool) -> Self {
        self.hold = hold;
        self
    }

    pub const fn with_swipe_up(mut self) -> Self {
        self.swipe_up = true;
        self
    }

    pub const fn with_swipe_down(mut self) -> Self {
        self.swipe_down = true;
        self
    }

    pub const fn with_swipe_right(mut self) -> Self {
        self.swipe_right = true;
        self
    }

    pub const fn with_footer(
        mut self,
        instruction: TString<'static>,
        description: Option<TString<'static>>,
    ) -> Self {
        self.footer_instruction = Some(instruction);
        self.footer_description = description;
        self
    }

    pub const fn with_chunkify(mut self, chunkify: bool) -> Self {
        self.chunkify = chunkify;
        self
    }

    pub const fn with_text_mono(mut self, text_mono: bool) -> Self {
        self.text_mono = text_mono;
        self
    }

    pub const fn with_page_limit(mut self, page_limit: Option<usize>) -> Self {
        self.page_limit = page_limit;
        self
    }

    pub const fn with_description_font(mut self, description_font: &'static TextStyle) -> Self {
        self.description_font = description_font;
        self
    }

    pub fn into_layout(
        self,
    ) -> Result<impl Component<Msg = FlowMsg> + Swipable + MaybeTrace, Error> {
        let paragraphs = ConfirmBlob {
            description: self.description.unwrap_or("".into()),
            extra: self.extra.unwrap_or("".into()),
            data: if self.data != Obj::const_none() {
                self.data.try_into()?
            } else {
                StrOrBytes::Str("".into())
            },
            description_font: &theme::TEXT_NORMAL,
            extra_font: &theme::TEXT_DEMIBOLD,
            data_font: if self.chunkify {
                let data: TString = self.data.try_into()?;
                theme::get_chunkified_text_style(data.len())
            } else if self.text_mono {
                &theme::TEXT_MONO
            } else {
                &theme::TEXT_NORMAL
            },
        }
        .into_paragraphs();

        let page = SwipeContent::new(SwipePage::vertical(paragraphs));
        let mut frame = Frame::left_aligned(self.title, page);
        if let Some(subtitle) = self.subtitle {
            frame = frame.with_subtitle(subtitle);
        }
        if self.menu_button {
            frame = frame.with_menu_button();
        }
        if self.cancel_button {
            frame = frame.with_cancel_button();
        }
        if let Some(instruction) = self.footer_instruction {
            frame = frame.with_footer(instruction, self.footer_description);
            frame = frame.with_swipe(Direction::Left, SwipeSettings::default());
        }

        if self.swipe_up {
            frame = frame.with_swipe(Direction::Up, SwipeSettings::default());
        }

        if self.swipe_down {
            frame = frame.with_swipe(Direction::Down, SwipeSettings::default());
        }

        if self.swipe_right {
            frame = frame.with_swipe(Direction::Right, SwipeSettings::default());
        }

        frame = frame.with_vertical_pages();

        Ok(frame.map(|msg| matches!(msg, FrameMsg::Button(_)).then_some(FlowMsg::Info)))
    }

    pub fn into_flow(self) -> Result<SwipeFlow, Error> {
        let paragraphs = ConfirmBlob {
            description: self.description.unwrap_or("".into()),
            extra: self.extra.unwrap_or("".into()),
            data: self.data.try_into()?,
            description_font: self.description_font,
            extra_font: &theme::TEXT_DEMIBOLD,
            data_font: if self.chunkify {
                let data: TString = self.data.try_into()?;
                theme::get_chunkified_text_style(data.len())
            } else if self.text_mono {
                &theme::TEXT_MONO
            } else {
                &theme::TEXT_NORMAL
            },
        }
        .into_paragraphs();

        flow::new_confirm_action_simple(
            paragraphs,
            ConfirmActionMenu::new(self.verb_cancel, self.info_button, self.verb_info),
            ConfirmActionStrings::new(
                self.title,
                self.subtitle,
                self.verb,
                self.prompt.then_some(self.title),
            ),
            self.hold,
            self.page_limit,
        )
    }
}

pub struct ShowInfoParams {
    title: TString<'static>,
    subtitle: Option<TString<'static>>,
    menu_button: bool,
    cancel_button: bool,
    footer_instruction: Option<TString<'static>>,
    footer_description: Option<TString<'static>>,
    chunkify: bool,
    swipe_up: bool,
    swipe_down: bool,
    items: Vec<(TString<'static>, TString<'static>), 4>,
}

impl ShowInfoParams {
    pub const fn new(title: TString<'static>) -> Self {
        Self {
            title,
            subtitle: None,
            menu_button: false,
            cancel_button: false,
            footer_instruction: None,
            footer_description: None,
            chunkify: false,
            swipe_up: false,
            swipe_down: false,
            items: Vec::new(),
        }
    }

    pub fn add(mut self, key: TString<'static>, value: TString<'static>) -> Option<Self> {
        if self.items.push((key, value)).is_ok() {
            Some(self)
        } else {
            None
        }
    }

    #[inline(never)]
    pub const fn with_subtitle(mut self, subtitle: Option<TString<'static>>) -> Self {
        self.subtitle = subtitle;
        self
    }

    #[inline(never)]
    pub const fn with_menu_button(mut self) -> Self {
        self.menu_button = true;
        self
    }

    #[inline(never)]
    pub const fn with_cancel_button(mut self) -> Self {
        self.cancel_button = true;
        self
    }

    #[inline(never)]
    pub const fn with_footer(
        mut self,
        instruction: TString<'static>,
        description: Option<TString<'static>>,
    ) -> Self {
        self.footer_instruction = Some(instruction);
        self.footer_description = description;
        self
    }

    pub const fn with_swipe_up(mut self) -> Self {
        self.swipe_up = true;
        self
    }

    pub const fn with_swipe_down(mut self) -> Self {
        self.swipe_down = true;
        self
    }

    #[inline(never)]
    pub fn into_layout(
        self,
    ) -> Result<impl Component<Msg = FlowMsg> + Swipable + MaybeTrace, Error> {
        let mut paragraphs = ParagraphVecShort::new();
        let mut first: bool = true;
        for item in self.items {
            // FIXME: padding:
            if !first {
                paragraphs.add(Paragraph::new::<TString<'static>>(
                    &theme::TEXT_SUB_GREY,
                    " ".into(),
                ));
            }
            first = false;
            paragraphs.add(Paragraph::new(&theme::TEXT_SUB_GREY, item.0).no_break());
            if self.chunkify {
                paragraphs.add(Paragraph::new(
                    theme::get_chunkified_text_style(item.1.len()),
                    item.1,
                ));
            } else {
                paragraphs.add(Paragraph::new(&theme::TEXT_MONO_GREY_LIGHT, item.1));
            }
        }

        let mut frame = Frame::left_aligned(
            self.title,
            SwipeContent::new(SwipePage::vertical(paragraphs.into_paragraphs())),
        );
        if let Some(subtitle) = self.subtitle {
            frame = frame.with_subtitle(subtitle);
        }
        if self.cancel_button {
            frame = frame
                .with_cancel_button()
                .with_swipe(Direction::Right, SwipeSettings::immediate());
        } else if self.menu_button {
            frame = frame
                .with_menu_button()
                .with_swipe(Direction::Left, SwipeSettings::default());
        }
        if let Some(instruction) = self.footer_instruction {
            frame = frame.with_footer(instruction, self.footer_description);
        }

        if self.swipe_up {
            frame = frame.with_swipe(Direction::Up, SwipeSettings::default());
        }

        if self.swipe_down {
            frame = frame.with_swipe(Direction::Down, SwipeSettings::default());
        }

        frame = frame.with_vertical_pages();

        Ok(frame.map(move |msg| {
            matches!(msg, FrameMsg::Button(_)).then_some(if self.cancel_button {
                FlowMsg::Cancelled
            } else {
                FlowMsg::Info
            })
        }))
    }
}
