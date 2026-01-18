use std::io; // input/output library
use std::cmp::Ordering; // comparison library
use rand::Rng; // random number generation library

fn main() {
    println!("Guess the number!");

    let secret_number = rand::thread_rng().gen_range(1..=100);

    // println!("The secret number is: {secret_number}");

    loop {
        println!("Please input your guess.");

        let mut guess = String::new();

        // Read user input from standard input
        io::stdin()
            .read_line(&mut guess)
            .expect("Failed to read line");

        // Shadowing the previous guess variable with a new one that has a different type
        let guess: u32 = match guess.trim().parse() {
            Ok(num) => num,
            Err(_) => continue,
        };

        println!("You guessed: {guess}");

        match guess.cmp(&secret_number) {
            Ordering::Less => println!("Too small!"),
            Ordering::Greater => println!("Too big!"),
            Ordering::Equal => {
                println!("You win!");
                break;
            }
        }
    }
}