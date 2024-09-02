use iced::advanced::widget::Operation;
use iced::advanced::{layout, mouse, widget::Tree, Layout, Widget};
use iced::{event, overlay, Element, Length, Renderer, Size, Theme, Vector};

pub struct FlowRow<'a, Message, Theme, Renderer> {
    children: Vec<Element<'a, Message, Theme, Renderer>>,
    spacing: f32,
}

impl<'a, Message> FlowRow<'a, Message, Theme, Renderer> {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
            spacing: 5.0,
        }
    }

    pub fn push<E>(mut self, child: E) -> Self
    where
        E: Into<Element<'a, Message>>,
    {
        self.children.push(child.into());
        self
    }

    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }
}

impl<'a, Message: 'a> Widget<Message, Theme, Renderer> for FlowRow<'a, Message, Theme, Renderer> {
    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Fill,
            height: Length::Shrink,
        }
    }

    fn layout(
        &self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = limits.width(Length::Fill).height(Length::Shrink);
        let mut current_row_width: f32 = 0.0;
        let mut current_row_height: f32 = 0.0;
        let mut rows = vec![];
        let mut current_row = vec![];

        if tree.children.len() != self.children.len() {
            tree.children = self.children.iter().map(|child| Tree::new(child)).collect();
        }

        for (child, child_tree) in self.children.iter().zip(tree.children.iter_mut()) {
            let child_layout = child.as_widget().layout(child_tree, renderer, &limits);
            let size = child_layout.size();

            if current_row_width + size.width > limits.max().width && !current_row.is_empty() {
                rows.push((current_row, current_row_height));
                current_row = vec![];
                current_row_width = 0.0;
                current_row_height = 0.0;
            }

            current_row.push((child_layout, size));
            current_row_width += size.width + self.spacing;
            current_row_height = current_row_height.max(size.height);
        }

        if !current_row.is_empty() {
            rows.push((current_row, current_row_height));
        }

        let mut total_height = 0.0;
        let mut layouts = vec![];

        for (row, height) in rows {
            let mut x_offset = 0.0;
            for (layout, size) in row {
                layouts.push(
                    layout::Node::with_children(Size::new(size.width, height), vec![layout])
                        .move_to(iced::Point::new(x_offset, total_height)),
                );
                x_offset += size.width + self.spacing;
            }
            total_height += height + self.spacing;
        }

        layout::Node::with_children(Size::new(limits.max().width, total_height), layouts)
    }

    // This method may need to be altered in the future. It's not currently used.
    fn operate(
        &self,
        state: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation<Message>,
    ) {
        operation.container(None, layout.bounds(), &mut |operation| {
            for ((child, state), layout) in self
                .children
                .iter()
                .zip(&mut state.children)
                .zip(layout.children())
            {
                child.as_widget().operate(
                    state,
                    layout.children().next().unwrap(),
                    renderer,
                    operation,
                );
            }
        });
    }

    fn draw(
        &self,
        state: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &iced::advanced::renderer::Style,
        layout: Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        for (child, (child_state, layout)) in self
            .children
            .iter()
            .zip(state.children.iter().zip(layout.children()))
        {
            child.as_widget().draw(
                child_state,
                renderer,
                theme,
                style,
                layout.children().next().unwrap(),
                cursor,
                viewport,
            );
        }
    }

    fn children(&self) -> Vec<Tree> {
        self.children.iter().map(|child| Tree::new(child)).collect()
    }

    fn on_event(
        &mut self,
        state: &mut Tree,
        event: iced::Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, Message>,
        viewport: &iced::Rectangle,
    ) -> event::Status {
        let mut status = event::Status::Ignored;

        let layout_bounds = layout.bounds();
        if cursor.is_over(layout_bounds) {
            for ((child, state), layout) in self
                .children
                .iter_mut()
                .zip(&mut state.children)
                .zip(layout.children())
            {
                let child_bounds = layout.bounds();
                if cursor.is_over(child_bounds) {
                    let child_status = child.as_widget_mut().on_event(
                        state,
                        event.clone(),
                        layout,
                        cursor,
                        renderer,
                        clipboard,
                        shell,
                        viewport,
                    );

                    status = child_status;

                    if status == event::Status::Captured {
                        break;
                    }
                }
            }
        }

        status
    }

    fn mouse_interaction(
        &self,
        state: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &iced::Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let mut interaction = mouse::Interaction::default();

        let layout_bounds = layout.bounds();
        if cursor.is_over(layout_bounds) {
            for ((child, state), layout) in self
                .children
                .iter()
                .zip(&state.children)
                .zip(layout.children())
            {
                let child_bounds = layout.bounds();
                if cursor.is_over(child_bounds) {
                    interaction = child
                        .as_widget()
                        .mouse_interaction(state, layout, cursor, viewport, renderer);

                    break;
                }
            }
        }

        interaction
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.children
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
            .find_map(|((child, child_tree), child_layout)| {
                child
                    .as_widget_mut()
                    .overlay(child_tree, child_layout, renderer, translation)
            })
    }
}

impl<'a, Message: 'a> From<FlowRow<'a, Message, Theme, Renderer>> for Element<'a, Message> {
    fn from(row: FlowRow<'a, Message, Theme, Renderer>) -> Self {
        Element::new(row)
    }
}
