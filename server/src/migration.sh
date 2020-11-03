# Scritp used to 'migrate' the DB


sqlite3 db.db "select * from users;" > users.txt
sqlite3 db.db "select epoch, winner_elo_diff, a.elo, b.elo, winner, loser from matches inner join users as a on a.id = winner inner join users as b on b.id = loser;" > matches.txt



#while IFS= read -r line 
#do
    #IFS='|' read -r -a array <<< "$line"
    #sqlite3 db.db "insert into users (id, name, elo) values (${array[0]}, \"${array[1]}\", ${array[2]})"
#done < "users.txt"

#while IFS= read -r line 
#do
    #IFS='|' read -r -a array <<< "$line"
    #sqlite3 db.db "insert into matches (epoch, winner, loser, elo_diff, winner_elo, loser_elo) values (${array[0]}, ${array[4]}, ${array[5]}, ${array[1]}, ${array[2]}, ${array[3]})"
#done < "matches.txt"
