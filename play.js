var fs = require('fs');
var text = fs.readFileSync('preprocessed-alice.txt', 'utf-8');
var matches = text.match(/(old|see|by|I'm|Said|said|An?|an?|The|the|With|with|Was|was|were|and) [A-Z]\w+([ \-][A-Z]\w+)?/g);
var matches2 = text.match(/[A-Z]\w+( [A-Z]\w+)? (replied)/g);
var uniqueMatches = [], set = new Set(), m;
for (var i = 0; i < matches.length; i++) {
    m = matches[i].replace(/.+? /, '');
    if (!set.has(m)) {
        set.add(m);
        uniqueMatches.push(m);
    }
}
for (var i = 0; i < matches2.length; i++) {
    m = matches2[i].replace(/ [^A-Z].+/, '');
    if (!set.has(m)) {
        set.add(m);
        uniqueMatches.push(m);
    }
}
uniqueMatches.sort();
for (var i = 0; i < uniqueMatches.length; i++) {
    console.log(uniqueMatches[i]);
}
console.log(uniqueMatches.length);