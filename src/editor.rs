use std::sync::Arc;

use nih_plug::prelude::Editor;
use nih_plug_vizia::{ViziaState, ViziaTheming, assets, create_vizia_editor, vizia::prelude::*, widgets::{ParamSlider, ResizeHandle}};

use crate::PluggedParams;

#[derive(Lens)]
struct Data {
    params: Arc<PluggedParams>,
}

impl Model for Data {}

pub(crate) fn default_state() -> Arc<ViziaState> {
    ViziaState::new(|| (400, 390))
}

pub(crate) fn create(params: Arc<PluggedParams>, editor_state: Arc<ViziaState>) -> Option<Box<dyn Editor>> {
    create_vizia_editor(editor_state, ViziaTheming::Custom, move |cx, _| {

        assets::register_noto_sans_light(cx);
        assets::register_noto_sans_thin(cx);

        Data {
            params: params.clone(),
        }.build(cx);

        VStack::new(cx, |cx| {
            Label::new(cx, "Plugged")
                .font_family(vec![FamilyOwned::Name(String::from(assets::NOTO_SANS))])
                .font_weight(FontWeightKeyword::Thin)
                .font_size(30.0)
                .height(Pixels(50.0))
                .child_top(Stretch(1.0))
                .child_bottom(Pixels(1.0));

            Label::new(cx, "Drive");
            ParamSlider::new(cx, Data::params, |params| &params.drive);
            
            Label::new(cx, "Mix");
            ParamSlider::new(cx, Data::params, |params| &params.mix);

            let diff = params.diff.lock().unwrap();

            Label::new(cx, format!("diff: {} {}", diff.first().unwrap(), diff.last().unwrap()).as_str());
        })
        .row_between(Pixels(0.0))
        .child_left(Stretch(1.0))
        .child_right(Stretch(1.0));

        ResizeHandle::new(cx);
    })
}