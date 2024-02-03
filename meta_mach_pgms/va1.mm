        ADR PROGRAM
PRIMARY
        ID 
        BF  A001 
        CL  'LD '
        CI 
        OUT 
A001 
        BT  A002 
        NUM 
        BF  A003 
        CL  'LDL'
        CI 
        OUT 
A003 
        BT  A002 
        TST  '('
        BF  A004 
        CLL EXP
        BE 
        TST  ')'
        BE 
A004 
A002 
        R 
TERM
        CLL PRIMARY
        BF  A005 
A006 
        TST  '*'
        BF  A007 
        CLL PRIMARY
        BE 
        CL  'MLT'
        OUT 
A007 
A008 
        BT  A006 
        SET 
        BE 
A005 
A009 
        R 
EXP1
        CLL TERM
        BF  A010 
A011 
        TST  '+'
        BF  A012 
        CLL TERM
        BE 
        CL  'ADD'
        OUT 
A012 
        BT  A013 
        TST  '-'
        BF  A014 
        CLL TERM
        BE 
        CL  'SUB'
        OUT 
A014 
A013 
        BT  A011 
        SET 
        BE 
A010 
A015 
        R 
EXP
        CLL EXP1
        BF  A016 
        TST  '.='
        BF  A017 
        CLL EXP1
        BE 
        CL  'EQU'
        OUT 
A017 
        BT  A018 
        SET 
        BF  A019 
A019 
A018 
        BE 
A016 
A020 
        R 
ASSIGNST
        CLL EXP
        BF  A021 
        TST  '='
        BE 
        ID 
        BE 
        CL  'ST '
        CI 
        OUT 
A021 
A022 
        R 
UNTILST
        TST  '.UNTIL'
        BF  A023 
        LB 
        GN1 
        OUT 
        CLL EXP
        BE 
        TST  '.DO'
        BE 
        CL  'BTP '
        GN2 
        OUT 
        CLL ST
        BE 
        CL  'B '
        GN1 
        OUT 
        LB 
        GN2 
        OUT 
A023 
A024 
        R 
CONDITIONALST
        TST  '.IF'
        BF  A025 
        CLL EXP
        BE 
        TST  '.THEN'
        BE 
        CL  'BFP'
        GN1 
        OUT 
        CLL ST
        BE 
        TST  '.ELSE'
        BE 
        CL  'B '
        GN2 
        OUT 
        LB 
        GN1 
        OUT 
        CLL ST
        BE 
        LB 
        GN2 
        OUT 
A025 
A026 
        R 
IOST
        TST  'EDIT'
        BF  A027 
        TST  '('
        BE 
        CLL EXP
        BE 
        TST  ','
        BE 
        SR 
        BE 
        CL  'EDT'
        CI 
        OUT 
        TST  ')'
        BE 
A027 
        BT  A028 
        TST  'PRINT'
        BF  A029 
        CL  'PNT'
        OUT 
A029 
A028 
        R 
IDSEQ1
        ID 
        BF  A030 
        LB 
        CI 
        OUT 
        CL  'BLK 1'
        OUT 
A030 
A031 
        R 
IDSEQ
        CLL IDSEQ1
        BF  A032 
A033 
        TST  ','
        BF  A034 
        CLL IDSEQ1
        BE 
A034 
A035 
        BT  A033 
        SET 
        BE 
A032 
A036 
        R 
DEC
        TST  '.REAL'
        BF  A037 
        CL  'B '
        GN1 
        OUT 
        CLL IDSEQ
        BE 
        LB 
        GN1 
        OUT 
A037 
A038 
        R 
BLOCK
        TST  '.BEGIN'
        BF  A039 
        CLL DEC
        BF  A040 
        TST  ';'
        BE 
A040 
        BT  A041 
        SET 
        BF  A042 
A042 
A041 
        BE 
        CLL ST
        BE 
A043 
        TST  ';'
        BF  A044 
        CLL ST
        BE 
A044 
A045 
        BT  A043 
        SET 
        BE 
        TST  '.END'
        BE 
A039 
A046 
        R 
ST
        CLL IOST
        BF  A047 
A047 
        BT  A048 
        CLL ASSIGNST
        BF  A049 
A049 
        BT  A048 
        CLL UNTILST
        BF  A050 
A050 
        BT  A048 
        CLL CONDITIONALST
        BF  A051 
A051 
        BT  A048 
        CLL BLOCK
        BF  A052 
A052 
A048 
        R 
PROGRAM
        CLL BLOCK
        BF  A053 
        CL  'HLT'
        OUT 
        CL  'END'
        OUT 
A053 
A054 
        R 
        END 
        
