use std::io::Write;

pub trait Serializable {
    /**
     * Serializes into bytes.
     */
    fn serializable_byte<W: Write> (&self, output: &mut W) -> Result<(), &'static str>;

    /**
     * Serializes into human readable output.
     */
    fn serializable_human(&self);
}
