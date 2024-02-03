        ADR PROGRAM
OUT1
        TST  '*1'
        BF  A001 
        CL  'GN1'
        OUT 
A001 
        BT  A002 
        TST  '*2'
        BF  A003 
        CL  'GN2'
        OUT 
A003 
        BT  A002 
        TST  '*'
        BF  A004 
        CL  'CI'
        OUT 
A004 
        BT  A002 
        SR 
        BF  A005 
        CL  'CL '
        CI 
        OUT 
A005 
A002 
        R 
OUTPUT
        TST  '.OUT'
        BF  A006 
        TST  '('
        BE 
A007 
        CLL OUT1
        BT  A007 
        SET 
        BE 
        TST  ')'
        BE 
A006 
        BT  A008 
        TST  '.LABEL'
        BF  A009 
        CL  'LB'
        OUT 
        CLL OUT1
        BE 
A009 
A008 
        BF  A010 
        CL  'OUT'
        OUT 
A010 
A011 
        R 
EX3
        ID 
        BF  A012 
        CL  'CLL'
        CI 
        OUT 
A012 
        BT  A013 
        SR 
        BF  A014 
        CL  'TST '
        CI 
        OUT 
A014 
        BT  A013 
        TST  '.ID'
        BF  A015 
        CL  'ID'
        OUT 
A015 
        BT  A013 
        TST  '.NUMBER'
        BF  A016 
        CL  'NUM'
        OUT 
A016 
        BT  A013 
        TST  '.STRING'
        BF  A017 
        CL  'SR'
        OUT 
A017 
        BT  A013 
        TST  '('
        BF  A018 
        CLL EX1
        BE 
        TST  ')'
        BE 
A018 
        BT  A013 
        TST  '.EMPTY'
        BF  A019 
        CL  'SET'
        OUT 
A019 
        BT  A013 
        TST  '$'
        BF  A020 
        LB 
        GN1 
        OUT 
        CLL EX3
        BE 
        CL  'BT '
        GN1 
        OUT 
        CL  'SET'
        OUT 
A020 
A013 
        R 
EX2
        CLL EX3
        BF  A021 
        CL  'BF '
        GN1 
        OUT 
A021 
        BT  A022 
        CLL OUTPUT
        BF  A023 
A023 
A022 
        BF  A024 
A025 
        CLL EX3
        BF  A026 
        CL  'BE'
        OUT 
A026 
        BT  A027 
        CLL OUTPUT
        BF  A028 
A028 
A027 
        BT  A025 
        SET 
        BE 
        LB 
        GN1 
        OUT 
A024 
A029 
        R 
EX1
        CLL EX2
        BF  A030 
A031 
        TST  '/'
        BF  A032 
        CL  'BT '
        GN1 
        OUT 
        CLL EX2
        BE 
A032 
A033 
        BT  A031 
        SET 
        BE 
        LB 
        GN1 
        OUT 
A030 
A034 
        R 
ST
        ID 
        BF  A035 
        LB 
        CI 
        OUT 
        TST  '='
        BE 
        CLL EX1
        BE 
        TST  ';'
        BE 
        CL  'R'
        OUT 
A035 
A036 
        R 
PROGRAM
        TST  '.SYNTAX'
        BF  A037 
        ID 
        BE 
        CL  'ADR'
        CI 
        OUT 
A038 
        CLL ST
        BT  A038 
        SET 
        BE 
        TST  '.END'
        BE 
        CL  'END'
        OUT 
A037 
A039 
        R 
        END 
        
