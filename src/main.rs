use std::collections::HashMap;
use std::io::{stdout, Write};
use std::time::Duration;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    execute,
    terminal::{self, ClearType},
    style::{Color, Print, SetForegroundColor, ResetColor},
};
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

// --- Basic Entities for Asteroids and Resources ---
#[derive(Debug, Clone)]
struct Asteroid {
    x: u16,
    y: u16,
}

#[derive(Debug, Clone)]
struct ResourceNode {
    x: u16,
    y: u16,
    kind: Resource,
}

// --- Rendering ---
fn render(ship: &Ship, asteroids: &[Asteroid], resources: &[ResourceNode], score: u32) {
    let mut stdout = stdout();
    execute!(stdout, terminal::Clear(ClearType::All), cursor::MoveTo(0, 0)).unwrap();

    // Draw border
    println!("╔════════════════════════════════════╗");
    for y in 0..15 {
        print!("║");
        for x in 0..34 {
            // Draw ship
            if x == ship.x && y == ship.y {
                print!(">A<");
                // Skip next 2 chars for ship width
                for _ in 0..2 { if x < 32 { print!(" "); } }
            }
            // Draw asteroids
            else if asteroids.iter().any(|a| a.x == x && a.y == y) {
                print!("O");
            }
            // Draw resources
            else if let Some(res) = resources.iter().find(|r| r.x == x && r.y == y) {
                match res.kind {
                    Resource::Iron => print!("*"),
                    Resource::Crystal => print!("♦"),
                    Resource::Gold => print!("$"),
                }
            }
            else {
                print!(" ");
            }
        }
        println!("║");
    }
    println!("╠════════════════════════════════════╣");
    print!("║ FUEL: ");
    let fuel_blocks = (ship.fuel / 10.0).round() as usize;
    for _ in 0..fuel_blocks { print!("█"); }
    for _ in fuel_blocks..10 { print!("░"); }
    print!("  CARGO: {}   SCORE: {} ║", ship.cargo.values().sum::<u32>(), score);
    println!();
    println!("╚════════════════════════════════════╝");
    stdout.flush().unwrap();
}

// --- Physics & Game Logic ---
fn physics_system(input: &InputEvent, ship: &mut Ship) {
    match input {
        InputEvent::Up if ship.y > 0 => ship.y -= 1,
        InputEvent::Down if ship.y < 14 => ship.y += 1,
        InputEvent::Left if ship.x > 0 => ship.x -= 1,
        InputEvent::Right if ship.x < 31 => ship.x += 1,
        _ => {}
    }
    // Fuel depletes over time
    ship.fuel = (ship.fuel - 0.5).max(0.0);
}

fn collision_system(ship: &Ship, asteroids: &[Asteroid]) -> bool {
    asteroids.iter().any(|a| a.x == ship.x && a.y == ship.y)
}

fn mining_system(input: &InputEvent, ship: &mut Ship, resources: &mut Vec<ResourceNode>) -> Option<Resource> {
    if let InputEvent::Mine = input {
        if let Some(idx) = resources.iter().position(|r| r.x == ship.x && r.y == ship.y) {
            let res = resources.remove(idx);
            *ship.cargo.entry(res.kind).or_insert(0) += 1;
            // Refuel if crystal
            if res.kind == Resource::Crystal {
                ship.fuel = (ship.fuel + 20.0).min(100.0);
            }
            return Some(res.kind);
        }
    }
    None
}

#[tokio::main]
async fn main() {
    // Setup terminal
    let mut stdout = stdout();
    terminal::enable_raw_mode().unwrap();
    execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide).unwrap();

    let mut ship = Ship::new();
    let mut asteroids = vec![
        Asteroid { x: 5, y: 5 },
        Asteroid { x: 20, y: 8 },
        Asteroid { x: 15, y: 12 },
    ];
    let mut resources = vec![
        ResourceNode { x: 8, y: 3, kind: Resource::Iron },
        ResourceNode { x: 25, y: 10, kind: Resource::Crystal },
        ResourceNode { x: 12, y: 7, kind: Resource::Gold },
    ];
    let mut score = 0;
    let mut tick: u32 = 0;
    let mut spawn_rate: u32 = 50; // Lower is faster

    // Show welcome screen
    execute!(stdout, terminal::Clear(ClearType::All), cursor::MoveTo(0, 0)).unwrap();
    println!("╔════════════════════════════════════╗");
    println!("║      RUSTY SPACE MINER            ║");
    println!("║------------------------------------║");
    println!("║  Use WASD to move, SPACE to mine   ║");
    println!("║  Avoid asteroids!                  ║");
    println!("║  Press Q to quit                   ║");
    println!("╚════════════════════════════════════╝");
    println!();
    println!("Press any key to start...");
    // Wait for any key
    loop {
        if event::poll(Duration::from_millis(10)).unwrap() {
            if let Event::Key(_) = event::read().unwrap() {
                break;
            }
        }
    }

    loop {
        render(&ship, &asteroids, &resources, score);

        let input = read_input().await;
        if let InputEvent::Quit = input {
            break;
        }

        physics_system(&input, &mut ship);

        // Asteroid Spawning
        tick += 1;
        if tick % spawn_rate == 0 {
            use rand::Rng;
            let mut rng = rand::thread_rng();
            let new_x = rng.gen_range(0..32);
            let new_y = rng.gen_range(0..15);
            asteroids.push(Asteroid { x: new_x, y: new_y });
        }
        // Increase Difficulty 
        if tick % 500 == 0 && spawn_rate > 10 {
            spawn_rate -= 5; // Asteroids spawn more frequently
        }

        if collision_system(&ship, &asteroids) || ship.fuel <= 0.0 {
            render(&ship, &asteroids, &resources, score);
            //This isn't working, I need to check this, I think it's something to do with the game loop ending and clearing the terminal
            println!("Game Over! Final Score: {}", score);
            break;
        }

        if let Some(_mined) = mining_system(&input, &mut ship, &mut resources) {
            score += 10;
        }

        tokio::time::sleep(Duration::from_millis(80)).await;
    }

    // Restore terminal
    execute!(stdout, cursor::Show, terminal::LeaveAlternateScreen).unwrap();
    terminal::disable_raw_mode().unwrap();
}