use std::cmp;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Symbol;

impl Symbol {
    pub fn get_or_store(string: &str) -> Self {
        Self
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct SrcData {
    lo: u32,
    hi: u32,
    ctxt: u32,
}

const ROOT_CTXT: u32 = 0x0;
pub const DUMMY_SRC_DATA: SrcData = SrcData { lo: 0, hi: 0, ctxt: 0 };

impl SrcData {
    fn new(lo: u32, hi: u32, ctxt: u32) -> SrcData {
        SrcData { lo, hi, ctxt }
    }

    pub fn with_root_ctxt(lo: BytePos, hi: BytePos) -> SrcData {
        SrcData { lo: lo.0, hi: hi.0, ctxt: ROOT_CTXT }
    }
    
    pub fn combine(&self, src_data: SrcData) -> SrcData {
        SrcData::new(cmp::min(self.lo, src_data.lo), cmp::max(self.hi, src_data.hi), self.ctxt)
    }
}

#[derive(Debug)]
pub struct GroupSrcIdx {
    open: SrcData,
    close: SrcData,
}

impl GroupSrcIdx {
    pub fn from_pair(open: SrcData, close: SrcData) -> GroupSrcIdx {
        GroupSrcIdx { open, close }
    }
}

macro_rules! impl_pos {
    (
        $(
            $(#[$attr:meta])*
            $vis_type:vis struct $type_name:ident ( $vis_member:vis $member_ty:ty );
        )*
    ) => {
        $(
            $(
                #[$attr]
            )*
            $vis_type struct $type_name ( $vis_member $member_ty );

            impl std::ops::Add for $type_name {
                type Output = $type_name;
                fn add(self, other: $type_name) -> Self::Output {
                    Self ( self.0 + other.0 )
                }
            }

            impl std::ops::Sub for $type_name {
                type Output = $type_name;
                fn sub(self, other: $type_name) -> Self::Output {
                    Self ( self.0 - other.0 )
                }
            }

            impl $type_name {
                pub fn from_usize(val: usize) -> Self {
                    Self ( val as $member_ty )
                }

                pub fn to_usize(self) -> usize {
                    self.0 as usize
                }
            }
        )*
    };
}


impl_pos!(
    #[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
    pub struct BytePos ( pub u32 );
);
