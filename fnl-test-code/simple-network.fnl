
network TestNetwork {    
    
    link L2 TestB TestA
    link L3 TestA TestB

    events e2, e3
    automata TestA {
        state b
        begin a
        trans ta {
            src a
            dst b
            input e2(L2)
            output e3(L3)
        }
        trans tb {
            src b
            dst a
            output e3(L3)
        }
    }

    automata TestB {
        begin a
        state b
        trans ta {
            src a
            dst b
            output e2(L2)
        }

        trans tb {
            src b 
            dst a
            input e3(L3)
        }

        trans tc {
            src b
            dst b
            input e3(L3)
        }

    }



}

request TestNetwork {
    space
}
