// visualizer: "Office TV" view of author activity (commit platform, pkgs/visualizer)
//! Activity visualizer (bevy): an "office TV" view where each author is a body
//! whose speed grows with their commit count, bouncing around an arena. Runs
//! headless here and prints final positions; the same systems drive the
//! windowed build.
//!
//!   visualizer alice=12 bob=5 carol=30

use bevy::prelude::*;

const ARENA: f32 = 100.0;
const DT: f32 = 1.0 / 60.0;

#[derive(Component, Debug, Clone, PartialEq)]
struct Author(String);

#[derive(Component, Debug, Clone, Copy, PartialEq)]
struct Position(Vec2);

#[derive(Component, Debug, Clone, Copy, PartialEq)]
struct Velocity(Vec2);

/// Speed scales with the square root of commits so prolific authors stand out
/// without leaving everyone else frozen.
fn speed_for(commits: u32) -> f32 {
    10.0 + 8.0 * (commits as f32).sqrt()
}

fn step(mut bodies: Query<(&mut Position, &mut Velocity)>) {
    for (mut pos, mut vel) in &mut bodies {
        pos.0 += vel.0 * DT;
        for axis in 0..2 {
            if pos.0[axis] < 0.0 || pos.0[axis] > ARENA {
                pos.0[axis] = pos.0[axis].clamp(0.0, ARENA);
                vel.0[axis] = -vel.0[axis];
            }
        }
    }
}

fn world(authors: &[(String, u32)]) -> App {
    let mut app = App::new();
    app.add_systems(Update, step);
    for (i, (name, commits)) in authors.iter().enumerate() {
        let angle = i as f32 * 2.4; // golden-angle spread
        let start = Vec2::splat(ARENA / 2.0) + Vec2::from_angle(angle) * 10.0;
        app.world_mut().spawn((
            Author(name.clone()),
            Position(start),
            Velocity(Vec2::from_angle(angle) * speed_for(*commits)),
        ));
    }
    app
}

fn main() {
    let authors: Vec<(String, u32)> = std::env::args()
        .skip(1)
        .filter_map(|a| {
            let (name, n) = a.split_once('=')?;
            Some((name.to_owned(), n.parse().ok()?))
        })
        .collect();
    let mut app = world(&authors);
    for _ in 0..600 {
        app.update();
    }
    let mut query = app.world_mut().query::<(&Author, &Position)>();
    for (author, pos) in query.iter(app.world()) {
        println!("{:<12} {:>7.2} {:>7.2}", author.0, pos.0.x, pos.0.y);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(authors: &[(String, u32)], ticks: usize) -> Vec<(String, Position, Velocity)> {
        let mut app = world(authors);
        for _ in 0..ticks {
            app.update();
        }
        let mut query = app.world_mut().query::<(&Author, &Position, &Velocity)>();
        query
            .iter(app.world())
            .map(|(a, p, v)| (a.0.clone(), *p, *v))
            .collect()
    }

    #[test]
    fn busier_authors_move_faster() {
        assert!(speed_for(100) > speed_for(10));
        let bodies = run(&[("busy".into(), 100), ("quiet".into(), 1)], 1);
        let speed = |name: &str| bodies.iter().find(|b| b.0 == name).unwrap().2.0.length();
        assert!(speed("busy") > speed("quiet"));
    }

    #[test]
    fn bodies_stay_inside_the_arena() {
        for (_, pos, _) in run(&[("a".into(), 400), ("b".into(), 50)], 1000) {
            assert!(
                (0.0..=ARENA).contains(&pos.0.x) && (0.0..=ARENA).contains(&pos.0.y),
                "{pos:?}"
            );
        }
    }
}
