use color_eyre::Result;
use dust_lang::{Map, Type, Value};
use lazy_static::lazy_static;
use ratatui::{
    prelude::Rect,
    style::{Color, Modifier, Style, Stylize},
    widgets::{Block, Borders},
    Frame,
};
use serial_int::SerialGenerator;
use std::{hash::Hash, sync::Mutex};
use tui_tree_widget::{Tree, TreeItem, TreeState};

use crate::{action::Action, components::Component};

lazy_static! {
    static ref ID_GENERATOR: Mutex<SerialGenerator<usize>> = Mutex::new(SerialGenerator::new());
}

fn create_tree_item<'a>(key: String, value: &Value) -> Result<TreeItem<'a, usize>> {
    let tree_item = match value {
        Value::List(list) => {
            let mut items = Vec::new();

            for (index, value) in list.items().iter().enumerate() {
                let item = create_tree_item(index.to_string(), value)?;

                items.push(item);
            }

            TreeItem::new(ID_GENERATOR.lock().unwrap().generate(), key, items)?
        }
        Value::Map(map) => {
            let mut items = Vec::new();

            for (key, (value, _)) in map.variables()?.iter() {
                let item = create_tree_item(key.to_string(), value)?;

                items.push(item);
            }

            TreeItem::new(ID_GENERATOR.lock().unwrap().generate(), key, items)?
        }
        Value::Function(_) => todo!(),
        Value::String(string) => TreeItem::new_leaf(
            ID_GENERATOR.lock().unwrap().generate(),
            format!("{key} <str> = {value}"),
        ),
        Value::Float(float) => TreeItem::new_leaf(
            ID_GENERATOR.lock().unwrap().generate(),
            format!("{key} <float> = {value}"),
        ),
        Value::Integer(integer) => TreeItem::new_leaf(
            ID_GENERATOR.lock().unwrap().generate(),
            format!("{key} <int> = {value}"),
        ),
        Value::Boolean(_) => todo!(),
        Value::Option(_) => todo!(),
    };

    Ok(tree_item)
}

pub struct MapDisplay<'a> {
    tree_state: TreeState<usize>,
    items: Vec<TreeItem<'a, usize>>,
}

impl<'a> MapDisplay<'a> {
    pub fn new(map: Map) -> Result<Self> {
        let tree_state = TreeState::default();
        let mut items = Vec::new();

        for (key, (value, _)) in map.variables()?.iter() {
            let item = create_tree_item(key.to_string(), value)?;

            items.push(item);
        }

        Ok(MapDisplay { tree_state, items })
    }
}

impl<'a> Component for MapDisplay<'a> {
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let tree = Tree::new(self.items.clone())?
            .block(Block::new().title("context").borders(Borders::ALL))
            .highlight_style(Style::new().add_modifier(Modifier::BOLD));

        frame.render_stateful_widget(tree, area, &mut self.tree_state);

        Ok(())
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::Up => self.tree_state.key_up(self.items.as_slice()),
            Action::Down => self.tree_state.key_down(&self.items),
            Action::Left => self.tree_state.key_left(),
            Action::Right => self.tree_state.key_right(),
            _ => {}
        }

        Ok(None)
    }
}
