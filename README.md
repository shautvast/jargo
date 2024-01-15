Exploring the need/possibility to move from maven/gradle to cargo.

That is, build tool for java taking inspiration from Cargo

And it's called Jargo. I do not wish to put a J in front of anything, as is the java tradition, 
but 'jargo' actually sounds kinda nice and it conveys pretty much what it is. 

It is NOT a new maven (not yet at least).

Basic premisses:
1. do NOT copy anything from the maven philosophy (phases, goals etc). Instead find out on the go what would be 
a good design
2. uses TOML 

Goals:
1. Simple management of (test) dependencies
2. ability to compile to jar files
3. ability to run unit tests

After this, we'll validate it's performance. If it's not faster/easier/better than maven, then abort

But instead, if it will save you time/resources/heart failure, then why not take this next level?

