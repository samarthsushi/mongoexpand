use mongoexpand::{MacroErrorT, crawler::Crawler, MacroProcessor, MacroEngine};

fn main() {
    let s = "$count { 
        { $field, $a }
        {
            { $group: {
                _id: '$field',
                a: { $sum: 1 }
            }
        }
    }";
    let q = "$count: { $branch }";
}
