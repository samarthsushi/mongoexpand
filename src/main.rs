use mongoexpand::{MacroErrorT, Crawler, MacroProcessor};

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
    let q = "$ount: { $branch, $cnt }";
    let mut c = Crawler::new(&s);
    let mut mp = match MacroProcessor::new(c.tokenize()) {
        Ok(x) => x,
        Err(e) =>{
            println!("{e}");
            std::process::exit(0);
        }
    };
    let res = mp.query(&q);
    println!("{}", res);
}
