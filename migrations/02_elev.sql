CREATE TABLE IF NOT EXISTS students (
    id 			TEXT 	NOT NULL,
    provenienta TEXT 	NOT NULL,
    judet 		TEXT    NOT NULL REFERENCES counties(code),
    
    medie_adm   REAL 	NOT NULL,
    medie_en 	REAL 	NOT NULL,
    medie_abs 	REAL 	NOT NULL,

    nota_ro 	 REAL 	NOT NULL,
    nota_mate 	 REAL 	NOT NULL,

    liceu 					TEXT 	NOT NULL,
    id_specializare 		INTEGER NOT NULL,
    specializare_display 	TEXT NOT NULL
);