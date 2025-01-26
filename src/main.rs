use mongoexpand::{crawler::Crawler, Macro};

fn main() {
    let s = "$count { 
        { $field, $a }
        {
            { $group: {
                _id: '$field',
                $a: { $sum: 1 }
            }
        }
    }";
    let q = "$count: { $branch }";
    let mut crawler = Crawler::new(&s);
    let mac = Macro::build(crawler.tokenize());
    println!("{:?}", mac);
}
