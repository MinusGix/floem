use std::{any::Any, f32::consts::E};

use glazier::{kurbo::Point, MouseEvent};

use crate::{
    app::AppContext,
    context::{EventCx, UpdateCx},
    event::Event,
    id::Id,
    view::{ChangeFlags, View},
};

pub struct Hover<V: View> {
    id: Id,
    child: V,
    on_hover_change: Box<dyn Fn(bool)>,
    is_hovered: bool,
}
impl<V: View> Hover<V> {
    fn update_hover(&mut self, cx: &mut EventCx, mouse_pos: Point) -> bool {
        let rect = cx.get_size(self.id).unwrap_or_default().to_rect();
        match (rect.contains(mouse_pos), self.is_hovered) {
            // The hover state did not change
            (true, true) | (false, false) => false,
            // Hover actually changed
            (is_now_hovered, _was_hovered) => {
                self.is_hovered = is_now_hovered;
                (self.on_hover_change)(self.is_hovered);
                true
            }
        }
    }
}

pub fn hover<V: View>(
    cx: AppContext,
    child: impl FnOnce(AppContext) -> V,
    on_hover_change: impl Fn(bool) + 'static,
) -> Hover<V> {
    let id = cx.new_id();
    let mut child_cx = cx;
    child_cx.id = id;
    let child = child(child_cx);
    Hover {
        id,
        child,
        on_hover_change: Box::new(on_hover_change),
        is_hovered: false,
    }
}

impl<V: View> View for Hover<V> {
    fn id(&self) -> Id {
        self.id
    }

    fn child(&mut self, id: Id) -> Option<&mut dyn View> {
        if self.child.id() == id {
            Some(&mut self.child)
        } else {
            None
        }
    }

    fn update(&mut self, cx: &mut UpdateCx, state: Box<dyn Any>) -> ChangeFlags {
        ChangeFlags::empty()
    }

    fn layout(&mut self, cx: &mut crate::context::LayoutCx) -> taffy::prelude::Node {
        cx.layout_node(self.id, true, |cx| vec![self.child.layout_main(cx)])
    }

    fn compute_layout(&mut self, cx: &mut crate::context::LayoutCx) {
        self.child.compute_layout_main(cx);
    }

    fn event(&mut self, cx: &mut EventCx, id_path: Option<&[Id]>, event: Event) -> bool {
        if id_path.is_none() {
            // only send event to child if id_path is_none,
            // because if id_path is_some, this event is destined to this view
            if self.child.event_main(cx, id_path, event.clone()) {
                return true;
            }
        }

        match &event {
            Event::MouseMove(event) | Event::MouseDown(event) | Event::MouseUp(event) => {
                self.update_hover(cx, event.pos)
            }
            _ => false,
        }
    }

    fn paint(&mut self, cx: &mut crate::context::PaintCx) {
        self.child.paint_main(cx);
    }
}
