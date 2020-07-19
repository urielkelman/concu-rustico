for run in $(seq 500)
do
  echo "4 jugadores, corrida $run"
  ./target/debug/rustico -p 4 -d debug
done

for run in $(seq 500)
do
  echo "6 jugadores, corrida $run"
  ./target/debug/rustico -p 6 -d debug
done

for run in $(seq 500)
do
  echo "8 jugadores, corrida $run"
  ./target/debug/rustico -p 6 -d debug
done

for run in $(seq 500)
do
  echo "10 jugadores, corrida $run"
  ./target/debug/rustico -p 10 -d debug
done

for run in $(seq 500)
do
  echo "12 jugadores, corrida $run"
  ./target/debug/rustico -p 12 -d debug
done
