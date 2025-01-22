use mongoexpand::{MacroErrorT, Crawler, MacroProcessor};
use std::collections::HashMap;

// static error_verbose_map: HashMap<MacroErrorT, &'static str> = HashMap::from([
//     (MacroErrorT::InvalidNumberOfDollars, "a literal should be preceded by just one dollar sign."),
//     (MacroErrorT::SuspendedDollar, "dollar is not followed by any literal. generics are required to have a name."),
//     (MacroErrorT::MissingComma, "expected comma here."),
//     (MacroErrorT::MissingParentheses, "expected parentheses here."),
//     (MacroErrorT::MissingName, "expected macro name here.")
// ]);


fn main() {
    let s = "$count { 
        { $field }
        {
            { $group: {
                _id: '$field',
                cnt: { $sum: 1 }
            }
        }
    }";
    let q = "$ount: { $branch }";
    let mut c = Crawler::new(&s);
    let mut mp = MacroProcessor::new(c.tokenize());
    mp.process();
    let res = mp.query(&q);
    println!("{:?}", res);
}
