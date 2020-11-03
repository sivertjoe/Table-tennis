sqlite3 db.db "select * from users;" > users.txt
sqlite3 db.db "select a.id, b.id, epoch from matches inner join users as a on winner = a.name inner join users as b on loser = b.name;" > matches.txt

touch "_db.db"


while IFS= read -r line 
do
    IFS='|' read -r -a array <<< "$line"
    sqlite3 _db.db "insert into users (id, name, elo) values (${array[0]}, ${array[1]}, ${array[2]})"
done < "users.txt"

while IFS= read -r line 
do
    IFS='|' read -r -a array <<< "$line"
    printf "${array[0]} ${array[1]} ${array[2]}\n"
    sqlite3 _db.db "insert into matches (winner, loser, winner_elo_diff, loser_elo_diff, epoch) values (${array[0]}, ${array[1]}, 0.0, 0.0, ${array[2]})"
done < "matches.txt"
