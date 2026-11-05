pub fn construct(bytes:&[u8])->base16ct::HexDisplay<'_>{base16ct::HexDisplay(bytes)}
pub fn project<'a>(view:base16ct::HexDisplay<'a>)->&'a[u8]{view.0}
pub fn construct_project(bytes:&[u8])->&[u8]{base16ct::HexDisplay(bytes).0}
