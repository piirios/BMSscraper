use serde::Deserialize;

/* liste des zones et region pour les BMS/BMR:
+--------+------+---------------------------------------+
| region | zone |               lieu                    |
+--------+------+---------------------------------------+
|   01   |  01  |   "Frontière belge - Baie de Somme"   |
+--------+------+---------------------------------------+
|   01   |  02  |     "Baie de Somme - La Hague"        |
+--------+------+---------------------------------------+
|   01   |  03  |       "La Hague – Penmarc'h"          |
+--------+------+---------------------------------------+
|   01   |  04  |       "Penmarc'h – Aiguillon"         |
+--------+------+---------------------------------------+
|   01   |  05  |    "Aiguillon - Frontière espagnole"  | 
+--------+------+---------------------------------------+
|   02   |  01  | "Frontière espagnole - Port Camargue" |
+--------+------+---------------------------------------+
|   02   |  02  |    "Port Camargue - Saint Raphaël"    |
+--------+------+---------------------------------------+
|   02   |  03  |     "Saint Raphaël – Menton"          |
+--------+------+---------------------------------------+
|   02   |  04  |              "Corse"                  |
+--------+------+---------------------------------------+


*/

#[derive(Deserialize)]
pub struct MFConfig {
    pub bmspath: String, //chemin pour le dossier où enregistrer les données des BMS
    pub bmrpath: String, //chemin pour le dossier où enregistrer les données des BMR
    pub region : u8,  //région pour le BMS/BMR (1 pour l'atlantique, 2 pour la méditerranée)
    pub zone: u8,     //zone pour le BMS/BMR de la côte
    pub want_bmr: bool,
    pub pretty: bool,
    pub run_every: String
}
