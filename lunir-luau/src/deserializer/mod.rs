// we should just make it a struct 
// it'd be nicer to work with than just a huge ass function
// y?
// like what, you get code in you get instructions out very simple
// thats what the AsRef<[u8]> is no?
// chunk isnt Vec<Instruction>?
// so what would it be
// ok

// well we also gotta define a LuauChunk struct 
// no no we don't return the instructions from this we return the chunk
// it's a structure 
// no no no not that I mean literal Lua chunks like functions
// A proto lemme show u in Prelude
pub fn deserialize(bytecode: impl AsRef<[u8]>) {
    
}