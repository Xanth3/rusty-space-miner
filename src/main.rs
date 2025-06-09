use std::collections::HashMap;
use std::time::Duration;
use crossterm::event::{self, Event, KeyCode};
use serde::{Serialize, Deserialize};
use tokio::task::yield_now;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
enum Resource {
    Iron,
    Crystal,
    Gold,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum Upgrade {
    Laser,
    Shields,
    Thrusters,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Ship {
    fuel: f32,
    cargo: HashMap<Resource, u32>,
    upgrades: Vec<Upgrade>,
    x: u16,
    y: u16,
}

impl Ship {
    fn new() -> Self {
        let mut cargo = HashMap::new();
        cargo.insert(Resource::Iron, 0);
        cargo.insert(Resource::Crystal, 0);
        cargo.insert(Resource::Gold, 0);
        Ship {
            fuel: 100.0,
            cargo,
            upgrades: Vec::new(),
            x: 10,
            y: 10,
        }
    }
}

#[derive(Debug, Clone)]
struct Rect {
    x: u16,
    y: u16,
    w: u16,
    h: u16,
}

fn check_collision(ship: &Rect, entity: &Rect) -> bool {
    ship.x < entity.x + entity.w &&
    ship.x + ship.w > entity.x &&
    ship.y < entity.y + entity.h &&
    ship.y + ship.h > entity.y
}

#[derive(Debug)]
enum InputEvent {
    Up,
    Down,
    Left,
    Right,
    Mine,
    Quit,
    None,
}

impl From<crossterm::event::KeyEvent> for InputEvent {
    fn from(key: crossterm::event::KeyEvent) -> Self {
        match key.code {
            KeyCode::Char('w') => InputEvent::Up,
            KeyCode::Char('a') => InputEvent::Left,
            KeyCode::Char('s') => InputEvent::Down,
            KeyCode::Char('d') => InputEvent::Right,
            KeyCode::Char(' ') => InputEvent::Mine,
            KeyCode::Char('q') => InputEvent::Quit,
            _ => InputEvent::None,
        }
    }
}

async fn read_input() -> InputEvent {
    loop {
        if event::poll(Duration::from_millis(10)).unwrap() {
            if let Event::Key(key) = event::read().unwrap() {
                return InputEvent::from(key);
            }
        }
        yield_now().await;
    }
}

#[tokio::main]
async fn main() {
    let mut ship = Ship::new();
    // TODO: Initialize asteroids, resources, game state

    loop {
        let input = read_input().await;
        match input {
            InputEvent::Quit => break,
            _ => {
                // TODO: Handle input, update game state, render
            }
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}