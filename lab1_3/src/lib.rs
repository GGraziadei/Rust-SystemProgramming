fn generate_delta(x: usize, y: usize, r: usize, c: usize) -> Vec<(i32, i32)> { //deltaX ; deltaY
    let mut delta = vec![];

    for i in -1..2 as i32 {
        //Escludi dal conteggio dei delta le righe di troppo
        if y as i32 +i >= r as i32 || y as i32 +i < 0 {
            continue;
        }
        for j in -1..2 as i32 {
            //Possibili motivi di esclusione del controllo
            if x as i32 +j >= c as i32 || x as i32 + j < 0 || (j == 0 && i == 0)  {
                continue;
            }
            delta.push((j ,i ));
        }
    }
    delta
}

pub fn annotate(minefield: &[&str]) -> Vec<String> {
    //Viene ricevuto un vettore di stringhe
    let r = minefield.len();
    if r == 0 {
        //Empty map
        return  vec![];
    }
    let c = minefield.first().unwrap().len();
    if c == 0 {
        //Empty map
        return  vec!["".to_string()];
    }

    let mut result = Vec::<String>::with_capacity(r);
    let matrix : Vec<char>= minefield.join("").chars().collect();

    for i in 0..r {
        let mut result_row = String::with_capacity(c);
        for j in 0..c{
            let value = matrix[c*i + j];
            if value == '*' {
                result_row.push(value);
                continue;
            }else{
                let mut mine_counter : u8 = 0;
                let delta = generate_delta(j,i, r,c);
                for d in delta {
                    let x = ( j as i32 + d.0 ) as usize;
                    let y = ( i as i32 + d.1 ) as usize ;
                    //Cast are also used for checking the boundary
                    if (y*c + x) > r*c || (y*c + x) < 0{
                        println!("UNBOUND");
                    }else{
                        println!("x: {} ; y: {}", x,y);
                    }
                    let test_field = matrix[y*c + x]; //
                    if test_field == '*' {
                        mine_counter += 1;
                    }
                }
                    let print_value = match mine_counter {
                        0 => ' ',
                        _ => char::from_digit(mine_counter as u32, 10).unwrap()
                    };

                    result_row.push(print_value );

            }
        }

        //Aggiungi la stringa al risultato
        result.push(result_row);
    }

    result
}

