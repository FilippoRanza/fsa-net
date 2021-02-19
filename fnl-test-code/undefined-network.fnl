/*
    This file contains a semantic error:
    'MissingNetwork' is not defined
*/
network CorrectDefinition {
    automata A {
        begin a1
        state a2
        state a3

        trans t1 a1 a2
        trans t2 a2 a3
        trans t3 a3 a1
    }
}

request CorrectDefinition {
    space
}


request MissingNetwork {
    space
}

network ExistingNetwork {
    automata A {
        begin a1
        state a2
        state a3

        trans t1 a1 a2
        trans t2 a2 a3
        trans t3 a3 a1
    }
}

request ExistingNetwork {
    space
}