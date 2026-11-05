use normalize_line_endings::normalized;

#[derive(Clone, Copy)]
pub struct Input { data: [char; 4], len: usize, pos: usize }
impl Iterator for Input {
    type Item = char;
    fn next(&mut self) -> Option<char> {
        if self.pos < self.len { let c=self.data[self.pos]; self.pos+=1; Some(c) } else { None }
    }
}

pub fn first(input: Input) -> Option<char> { normalized(input).next() }

#[cfg(kani)]
mod verification {
    use super::*;
    // Independent eager specification: replace CR by LF; omit LF immediately after CR.
    fn reference(data:[char;4], len:usize)->([char;4],usize) {
        let mut out=['\0';4]; let mut n=0; let mut i=0;
        while i<len {
            let c=data[i];
            if c!='\n' || i==0 || data[i-1]!='\r' {
                out[n]=if c=='\r' {'\n'} else {c}; n+=1;
            }
            i+=1;
        }
        (out,n)
    }
    #[kani::proof]
    #[kani::unwind(8)]
    fn original_bounded_normalization() {
        let data:[char;4]=kani::any(); let len:usize=kani::any();
        kani::assume(len<=4);
        let (expected,n)=reference(data,len);
        let mut actual=normalized(Input{data,len,pos:0});
        assert_eq!(actual.size_hint(),(0,None));
        let mut output=['\0';4]; let mut i=0;
        while i<n { output[i]=actual.next().expect("reference output exists"); i+=1; }
        assert_eq!(actual.next(),None);
        assert_eq!(actual.next(),None);
        assert_eq!(output,expected);
        assert!(n<=len);
        let mut j=0; while j<n { assert_ne!(output[j],'\r'); j+=1; }
        let mut twice=normalized(Input{data:output,len:n,pos:0});
        let mut k=0; while k<n { assert_eq!(twice.next(),Some(output[k])); k+=1; }
        assert_eq!(twice.next(),None);
    }
}
