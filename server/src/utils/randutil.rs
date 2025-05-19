use rand::Rng;

pub fn random_organization_name() -> String {
    let words: Vec<&str> = include_str!("../resources/4-letter-words.txt")
        .lines()
        .collect();
    let mut rng = rand::thread_rng();
    let (a, b): (usize, usize) = (
        rng.gen::<usize>() % words.len(),
        rng.gen::<usize>() % words.len(),
    );

    format!("{}-{}", words[a], words[b])
}
