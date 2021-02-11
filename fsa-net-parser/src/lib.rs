#[macro_use]
extern crate lalrpop_util;
lalrpop_mod!(fsa_net_lang);

mod syntax_tree;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
