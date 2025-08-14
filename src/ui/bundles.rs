use crate::ui::components::ButtonConfig;
use bevy::color::Color;
use bevy::prelude::*;

#[derive(Bundle)]
pub struct LabelBundle {
    pub text: Text,
    pub font: TextFont,
    pub color: TextColor,
    pub node: Node,
}

impl LabelBundle {
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
    pub border_radius: BorderRadius,
    pub interaction: Interaction,
    pub config: ButtonConfig,
}

impl SmallButtonBundle {
    pub fn new(size: f32, normal_color: Color, hover_color: Color, pressed_color: Color) -> Self {
        Self {
            button: Button,
            node: Node {
                width: Val::Px(size),
                height: Val::Px(size),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            border_radius: BorderRadius::all(Val::Px(5.0)),
            interaction: Interaction::None,
            config: ButtonConfig {
                normal_color,
                hover_color,
                pressed_color,
            },
        }
    }
}
