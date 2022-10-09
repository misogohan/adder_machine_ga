use rand::{prelude::SliceRandom, Rng};

struct BoolStack(Vec<bool>);
impl BoolStack {
    fn new() -> Self {
        Self(vec![])
    }
    fn pop(&mut self) -> bool {
        self.0.pop().unwrap_or(false)
    }
    fn push(&mut self, b: bool) {
        self.0.push(b)
    }
}
fn random_name() -> String {
    let consonant = "bcdfghjklmnpqrstvwxyz".chars().collect::<Vec<_>>();
    let vowel = "aeiou".chars().collect::<Vec<_>>();
    [
        consonant[(rand::random::<f64>() * 21.0) as usize],
        vowel[(rand::random::<f64>() * 5.0) as usize],
        consonant[(rand::random::<f64>() * 21.0) as usize],
    ]
    .iter()
    .collect()
}

#[derive(Debug)]
struct Machine {
    name: String,
    genome: [u8; 16],
}
impl Machine {
    fn from(genome: [u8; 16]) -> Self {
        Self {
            genome,
            name: random_name(),
        }
    }
    fn new_random(rng: &mut impl Rng) -> Self {
        let mut genome = [0; 16];
        for i in 0..16 {
            genome[i] = rng.gen();
        }
        Self::from(genome)
    }
    fn run(&self, arg0: bool, arg1: bool) -> (bool, bool) {
        let mut register = vec![arg0, arg1, false];
        let mut stack = BoolStack::new();
        for byte in self.genome {
            match byte & 0b0111 {
                0b0000 => {}
                0b0001 => {
                    let a = stack.pop();
                    let b = stack.pop();
                    stack.push(!(a & b));
                }
                0b0010 => stack.push(register[0]),
                0b0011 => register[0] = stack.pop(),
                0b0100 => stack.push(register[1]),
                0b0101 => register[1] = stack.pop(),
                0b0110 => stack.push(register[2]),
                0b0111 => register[2] = stack.pop(),
                _ => unreachable!(),
            }
        }
        let c = stack.pop();
        let s = stack.pop();
        (s, c)
    }
    fn pure(&self) -> Self {
        Self {
            name: self.name.clone(),
            genome: self.genome,
        }
    }
    fn mutate_copy_error(&self, rng: &mut impl Rng) -> Self {
        let mut new = Self::from(self.genome);
        let i = rng.gen::<usize>() % 16;
        let filter = rng.gen::<u8>();
        new.genome[i] ^= filter;
        new
    }
    fn mutate_shift(&self, rng: &mut impl Rng) -> Self {
        let mut new = Self::from(self.genome);
        let i = rng.gen::<usize>() % 16;
        new.genome[i] >>= 1;
        new
    }
    fn mutate_rotate(&self, rng: &mut impl Rng) -> Self {
        let mut new = Self::from(self.genome);
        if rng.gen::<f32>() < 0.5 {
            new.genome.rotate_right(1);
        } else {
            new.genome.rotate_left(1);
        }
        new
    }
    fn mutate_replicate(&self, rng: &mut impl Rng) -> Self {
        let mut new = Self::from(self.genome);
        let i = rng.gen::<usize>() % 16;
        let j = rng.gen::<usize>() % 16;
        new.genome[j] = new.genome[i];
        new
    }
}

fn test_half_add(machine: &Machine) -> f64 {
    let cases = vec![
        ((false, false), (false, false)),
        ((false, true), (true, false)),
        ((true, false), (true, false)),
        ((true, true), (false, true)),
    ];
    let n = cases.len();

    let mut score = 0.0;
    for ((arg0, arg1), (s, c)) in cases {
        let (s_calced, c_calced) = machine.run(arg0, arg1);
        // println!("{s} {c}, {s_calced} {c_calced}");
        if s_calced == s {
            score += 0.75;
        }
        if c_calced == c {
            score += 0.25;
        }
    }
    score / n as f64
}

fn main() {
    let n = 32;

    let mut rng = rand::thread_rng();

    let mut generation = (0..n)
        .map(|_| Machine::new_random(&mut rng))
        .collect::<Vec<_>>();

    let mut count = -1;
    let machine = loop {
        count += 1;

        let mut scores = generation
            .iter()
            .map(|m| (test_half_add(m), m))
            .collect::<Vec<_>>();
        scores.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        scores.reverse();

        if count % 100 == 0 {
            if count != 0 {
                print!("\x1b[4A\x1b[0J");
            }
            println!("{count}th generation");
            print!(" name");
            for name in scores.iter().map(|(_, m)| &m.name) {
                print!(" {name:>6}");
            }
            print!("\nscore");
            for (score, _) in &scores {
                print!(" {score:.4}");
            }
            print!("\n");

            let (best_score, best_machine) = scores[0];
            if best_score == 1.0 {
                break best_machine;
            }
            println!("{:?}", best_machine);
        }

        std::thread::sleep_ms(1);
        let (_, best_machine) = scores[0];
        generation = (0..n)
            .map(|i| match (i * 4 + n - 1) / n {
                0 => best_machine.pure(),
                1 => best_machine.mutate_copy_error(&mut rng),
                2 => best_machine.mutate_shift(&mut rng),
                3 => best_machine.mutate_rotate(&mut rng),
                4 => best_machine.mutate_replicate(&mut rng),
                _ => unreachable!(),
            })
            .collect();
        generation.shuffle(&mut rng);
    };
    println!("{:?}", machine);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn half_add() {
        assert_eq!(
            1.0,
            test_half_add(&Machine::from([
                0b0010, 0b0100, 0b0001, 0b0111, 0b0010, 0b0110, 0b0001, 0b0100, 0b0110, 0b0001,
                0b0001, 0b0110, 0b0110, 0b0001, 0, 0
            ]))
        )
    }
}
