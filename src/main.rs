use url::Url;
use serde_json::Value;
use tungstenite::{connect, Message};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};
use std::io::{Error, ErrorKind};
use soloud::*;



fn main()->Result<(), Error> {


    // The buy and sell threshold triggering a specific an alert sound

    let buy_amount_limit:f32=10.0;
    let sell_amount_limit:f32= -10.0;


    // Make sure to change the path to the alert .wav files before turning withsound to true
    let withsound=false;
    let with_csv_saving=false;


    let sl = Soloud::default().unwrap();
    let mut wav = audio::Wav::default();
    let mut wav2 = audio::Wav::default();

    //wav.load_mem(include_bytes!("path_to_the_alert_folders/alert1.wav")).unwrap();
    //wav2.load_mem(include_bytes!("path_to_the_alert_folders/alert2.wav")).unwrap();
    



    let starting_timestamp: i64 = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_secs() as i64;


    let mut all_trades: Vec<String>=Vec::new();

    let (mut socket, response) = connect(
        Url::parse("wss://stream.binance.com/ws/BTCUSDT@aggTrade").unwrap()
    ).expect("Can't connect");


    socket.write_message(Message::Text(r#"{
    "method": "SUBSCRIBE",
    "params":
    [
    "btcfdusd@aggTrade"
    ],
    "id": 1
    }"#.into()));

    let mut net_buyer_amount:f32=0.0;
 

    loop {
        let msg = socket.read_message().expect("Error reading message");
        let str_message=&msg.into_text().unwrap();
        let mut object: Value = serde_json::from_str(str_message).unwrap();


        //Sometimes the message returned is an error so we're filtering with messages
        // returning a trade characteristic
        if str_message.contains("T"){
            
            
            let evt_time=object.get("T").unwrap().as_i64().unwrap();
            let evt_time_s=evt_time.to_string();
            let price=object.get("p").unwrap().as_str().unwrap();
            
            let mut quantity=object.get("q").unwrap().as_str().unwrap();
            let mut sign="".to_owned();
            let is_a_sell=object.get("m").unwrap().as_bool().unwrap();
            
            
            if is_a_sell{
                sign="-".to_string();
            }
            sign.push_str(quantity);
            let data_ts=evt_time_s+","+price+","+&sign;
            all_trades.push(data_ts);
            //println!("Received: {}", data_ts);



            let amount:f32=sign.parse::<f32>().unwrap();
            net_buyer_amount=net_buyer_amount+amount;

            if withsound{


                        if net_buyer_amount>buy_amount_limit{
                            sl.play(&wav);
                            net_buyer_amount=0.0;
                        }
                        

                        if net_buyer_amount< sell_amount_limit{

                            sl.play(&wav2);
                            net_buyer_amount=0.0;

                        }
            }
            

            if all_trades.len()>1000{

                    
                    let current_timestamp: i64 = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64;


                    if with_csv_saving{

                        //That block is for writing the info from ram into a csv file
                        let mut path_report:String="path_to_the_saving_folder\\trades_ws_BTCFDUSD_".to_owned();
                        path_report.push_str(&current_timestamp.to_string());
                        path_report.push_str(&".csv");

                        let mut datarrt :String = "time,price,amount".to_owned();
                        datarrt.push_str(&"\n");

                        for element in 0..all_trades.len(){
                            
                            let joined: String = all_trades[0].to_owned();
                            
                            datarrt.push_str(&joined);
                            datarrt.push_str(&"\n");
                            all_trades.remove(0);
                        }

                        fs::write(path_report, datarrt).expect("Unable to write file");


                    }else{

                        // if no csv saving we still need to empty the list of trades for RAM efficiency
                        for element in 0..all_trades.len(){
                            all_trades.remove(0);
                        }

                    }


                    // The websocket will close the connection every 24h so raising an exception is a dirty yet
                    // efficient way to trigger a restart of the code after encapsulating this code in a process manager
                    if current_timestamp-starting_timestamp>22*60*60{
                        return Err(Error::new(ErrorKind::NotFound, format!("FileNotFoundError: {}", "END OF DAY")));
                    }


            }else{
                println!("Net amount bought : {}",net_buyer_amount);
            }

        }
    }
}


