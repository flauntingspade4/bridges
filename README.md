So a year ago, for my emc I wrote a simple bitmap file format, and specifically said as a comment for no-one to use it outside this project
Little did I know, Will then helped Josh Smart's ray-tracer, as previously he was saving bitmap images as text files. Will yoinked and improved my code (Which had previously only been used by grey-scale images)
Two days later, Josh Smart found a bug where it was RGB instead of BGR
Then, SIX MONTHS LATER Will was writing a new ray-tracer, and has just spent 2 hours trying to fix a bug where mysteriously the red and blue channels were flipped

In conclusion: The bmp spec is wrong 

@w-henderson @joshua-smart
