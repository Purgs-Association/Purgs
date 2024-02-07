use purgs::parse;

fn main() {
    println!("{:#?}", parse(include_str!("double_top.pug")));
}
