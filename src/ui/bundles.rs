use bevy::color::Color;
use bevy::prelude::*;

#[derive(Bundle)]
pub struct UILabelBundle {
    pub text: Text,
    pub font: TextFont,
    pub color: TextColor,
    pub node: Node,
}

impl UILabelBundle {
    pub fn new(text: &str, font_size: f32, color: Color) -> Self {
        Self {
            text: Text::new(text),
            font: TextFont {
                font_size,
                ..default()
            },
            color: TextColor(color),
            node: Node::default(),
        }
    }

    pub fn with_margin(mut self, margin: UiRect) -> Self {
        self.node.margin = margin;
        self
    }

    pub fn with_node(mut self, node: Node) -> Self {
        self.node = node;
        self
    }
}

#[derive(Bundle)]
pub struct SmallButtonBundle {
    pub button: Button,
    pub node: Node,
    pub background: BackgroundColor,
    pub border_radius: BorderRadius,
    pub interaction: Interaction,
}

impl SmallButtonBundle {
    pub fn new(size: f32, background_color: Color) -> Self {
        Self {
            button: Button,
            node: Node {
                width: Val::Px(size),
                height: Val::Px(size),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            background: BackgroundColor(background_color),
            border_radius: BorderRadius::all(Val::Px(5.0)),
            interaction: Interaction::None,
        }
    }
}
