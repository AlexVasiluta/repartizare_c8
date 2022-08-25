CREATE TABLE IF NOT EXISTS specializari (
    id 			INTEGER NOT NULL,
    name 		TEXT 	NOT NULL,
    liceu 		TEXT 	NOT NULL,
    mediu 		TEXT 	NOT NULL,
    judet 		TEXT    NOT NULL REFERENCES counties(code),

    specializare TEXT 	NOT NULL,
    bilingv 	INTEGER NOT NULL,

    locuri 		INTEGER NOT NULL,
    ocupate 	INTEGER NOT NULL,

    profil 		TEXT 	NOT NULL,
    filiera 	TEXT 	NOT NULL,

    ultima_medie REAL 	NOT NULL,
    ultima_medie_ant REAL NOT NULL
);