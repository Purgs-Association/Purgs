use purgs::parse;

fn main() {
    println!("{:#?}", parse(include_str!("tag.pug")));
}
