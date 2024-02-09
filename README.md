**Jargo**

----

**edit:**

work is stalled, since the xmlparser (hard-xml) does not allow xml entities that are not known upfront, ie the &lt;properties> element has them for every property.
I don't even think it should support them and/or the regular java libs like jaxb would do that.
So ... thinking of forking, but the code is really hard (declarative macro's) and not documented...

background: we need to parse maven xml to be able to use remote maven repositories.

----
 
An experimental build tool for Java taking inspiration from Cargo.

And it's called *Jargo*. I do not wish to put a J in front of anything, as is the java tradition, 
but 'jargo' actually sounds kinda nice and it conveys pretty much what it is. 

It is NOT a new maven (not yet at least). That's the reason it's not called 'raven.'

Basic premisses:
1. written in Rust
2. does NOT copy anything from the maven philosophy (phases, goals etc). Instead find out on the go what would be 
a good design. _That said, some things are just good to keep using, such as the default project structure._
3. configured in TOML. ie. no XML, **yay!**, AND no Turing-completeness (groovy/kotlin in gradle), **yay2!!**

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

_While that would (in theory) make migration a no-brainer, it seems too ambitious based on what I've seen of the maven 
codebase. Other than that you will most likely run into onforeseen issues while migrating this way, because this or 
that is subtly different here and there. Better avoid the promise of easy migration altogether._


