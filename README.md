**Jargo**

Exploring the need/possibility to move from maven/gradle to cargo.

That is, build tool for java taking inspiration from Cargo

And it's called Jargo. I do not wish to put a J in front of anything, as is the java tradition, 
but 'jargo' actually sounds kinda nice and it conveys pretty much what it is. 

It is NOT a new maven (not yet at least). That's the reason it's not called 'raven.'

Basic premisses:
1. written in Rust
2. does NOT copy anything from the maven philosophy (phases, goals etc). Instead find out on the go what would be 
a good design
3. uses TOML

see [tests/sample_project/Jargo.toml](https://github.com/shautvast/jargo/blob/main/tests/sample_project/Jargo.toml) to get an impression of what that looks like.

Goals:
1. Simple management of (test) dependencies, using existing maven repositories
2. ability to compile to jar files
3. ability to run unit tests

After this, we'll validate it's performance. If it's not faster/easier/better than maven, then abort

But instead, if it will save you time/resources/heart failure, then why not take this next level?

4. upload to maven repo's
5. plugin mechanism for specific goals (code generation, javadoc, etc). 
6. migrating from maven in actual projects


Questions:
1. Why?

_Every tool is currently being rewritten in rust._ And for good reason!

2. Why not create a drop-in replacement for maven written in rust?

_While that would make migration a no-brainer, it seems too ambitious based on what I've seen of the maven 
codebase_


