use cfgrammar::yacc::YaccKind;
use lrlex::CTLexerBuilder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    CTLexerBuilder::new()
        .lrpar_config(|ctp| {
            ctp.yacckind(YaccKind::Grmtools)
                .grammar_in_src_dir("cond.y")
                .unwrap()
        })
        .lexer_in_src_dir("cond.l")?
        .build()?;
        CTLexerBuilder::new()
        .lrpar_config(|ctp| {
            ctp.yacckind(YaccKind::Grmtools)
                .grammar_in_src_dir("validate.y")
                .unwrap()
        })
        .lexer_in_src_dir("validate.l")?
        .build()?;
    Ok(())
}
