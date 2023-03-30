%start Program
%left '||'
%right '&&'

%%
Program -> anyhow::Result<()>:
  { Ok(()) }
  ;
%%

use crate::*;
