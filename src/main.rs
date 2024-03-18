use crossterm::{
    cursor, execute,
    style::{self, Stylize},
    terminal::{size, EnterAlternateScreen, LeaveAlternateScreen, SetSize},
    QueueableCommand,
};
use nanorand::{Rng, WyRand};
use std::env;
use std::io::{stdout, Write};
use std::ops::Div;
use std::{thread, time};

fn main() {
    let argument: Vec<String> = env::args().skip(1).collect();
    if argument.len() != 3 || argument[0] == "help" {
        eprintln!("Arguments required are: \n<cells> (800-2200) \t<delay> (80-750 millis) \t<generations> (minimum 10)");
    } else {
        // argument parsing
        let alive_cells = argument[0]
            .parse::<usize>()
            .expect("parsing of cells failed");
        let tdelay = argument[1].parse::<u64>().expect("parsing of dealy failed");
        let maxgens = argument[2].parse::<usize>().expect("parsing gen failed");
        let delay = time::Duration::from_secs(1 / 60);
        let (xmax, ymax) = {
            let (xmax, ymax) = size().expect("Failed at getting size");
            (xmax as usize, ymax as usize)
        };
        let max = (xmax, ymax);

        // create first gen of the cells
        let mut cells = vec![vec![(true, false); xmax]; ymax];
        first_gen(&mut cells, alive_cells, max);

        // set terminal
        execute!(stdout(), EnterAlternateScreen).expect("Cannot spawn new window");
        execute!(stdout(), cursor::Hide).expect("Failed to hide cursor");
        execute!(stdout(), SetSize(xmax as u16, ymax as u16)).expect("Cannot resize window");

        // loop
        let mut gens = 1;
        while gens <= maxgens {
            cells = next_generation(cells, max);
            thread::sleep(delay);
            display(&cells, gens, max);
            gens += 1;
        }

        //reset terminal
        execute!(stdout(), SetSize(xmax as u16, ymax as u16)).expect("Cannot resize window");
        execute!(stdout(), cursor::Show).expect("Failed to show cursor");
        execute!(stdout(), LeaveAlternateScreen).expect("Cannot exit window");
    }
}

fn display(cells: &Vec<Vec<(bool, bool)>>, gen: usize, (xmax, ymax): (usize, usize)) {
    let mut stdout = stdout();

    stdout
        .queue(cursor::MoveTo(3, 0))
        .expect("paniced which setting gen")
        .queue(style::Print(format!("{:03}", gen)))
        .expect("paniced which setting gen");

    for y in 1..ymax - 1 {
        for x in 1..xmax - 1 {
            if cells[y][x].0 {
                if cells[y][x].1 {
                    stdout
                        .queue(cursor::MoveTo(x as u16, y as u16))
                        .expect("cursor jammed !")
                        .queue(style::PrintStyledContent("O".magenta()))
                        .expect("problem in drawing");
                } else {
                    stdout
                        .queue(cursor::MoveTo(x as u16, y as u16))
                        .expect("cursor jammed !")
                        .queue(style::PrintStyledContent(" ".white()))
                        .expect("problem in drawing");
                }
            }
        }
    }
    stdout.flush().expect("flushing failed");
}

fn first_gen(cells: &mut Vec<Vec<(bool, bool)>>, alive_cells: usize, (xmax, ymax): (usize, usize)) {
    let mut rnd = WyRand::new();
    let one_fourth = |x: usize| x.div(4);
    let three_forth = |x| 3 * one_fourth(x);
    for _ in 0..alive_cells {
        let (x, y) = (
            rnd.generate_range(one_fourth(xmax)..three_forth(xmax)),
            rnd.generate_range(one_fourth(ymax)..three_forth(ymax)),
        );
        cells[y][x].1 = true;
    }
}

fn next_generation(cells: &mut Vec<Vec<(bool, bool)>>, (xmax, ymax): (usize, usize)) {
    let mut new_cells = [[(false, false); xmax]; ymax];
    for y in 0..ymax {
        for x in 0..xmax {
            let alive_neighbors = alive_neighbors(&cells, (y, x), (xmax, ymax));
            new_cells[y][x] = if cells[y][x].1 {
                match alive_neighbors {
                    0..=1 => (true, false), // dead
                    2..=3 => (true, true),  // alive
                    4..=8 => (true, false), // overpopulation
                    _ => panic!("phantom neighbors"),
                }
            } else {
                match alive_neighbors {
                    3 => (true, true),
                    0..=2 => (true, false),
                    4..=8 => (true, false),
                    _ => panic!("phantom neighbors"),
                }
            }
        }
    }
    new_cells
}

fn alive_neighbors(
    cells: &Vec<Vec<(bool, bool)>>,
    coordinate: (usize, usize),
    (xmax, ymax): (usize, usize),
) -> u8 {
    let adjacent: [(i16, i16); 8] = [
        (0, 1),
        (0, -1),
        (1, 0),
        (-1, 0),
        (-1, 1),
        (1, -1),
        (1, 1),
        (-1, -1),
    ];

    let (y, x) = coordinate;
    let alive_neighbors = adjacent
        .map(|(y0, x0)| (y0 + y as i16, x0 + x as i16))
        .into_iter()
        .filter(|(y, x)| (*y as usize) < ymax && (*x as usize) < xmax)
        .filter(|(y, x)| cells[*y as usize][*x as usize].1)
        .count();

    alive_neighbors as u8
}
