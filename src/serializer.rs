use std::io::Write;

pub trait Serializable {
    /**
     * Serializes into bytes.
     */
    fn serializable_byte<W: Write>(&self, output: &mut W) -> std::io::Result<()>;
}
