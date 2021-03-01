/*
    This code requires to compute a linspace where one 
    observability label is a state name
*/

request Test {
    // s0 is a state
    linspace o1, o5, s0
}

network Test {

    obs o1, o2, o3, o4, o5, o6

    automata A1 {
        begin s0
        state s1
        state s2
    
        trans t1 s0 s1
        trans t2 s1 s2
        trans t3 s2 s0
    }

}

