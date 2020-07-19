for run in $(seq 500)
do
  ./target/debug/rustico -p 4 -d debug
done

for run in $(seq 500)
do
  ./target/debug/rustico -p 6 -d debug
done

for run in $(seq 500)
do
  ./target/debug/rustico -p 10 -d debug
done

for run in $(seq 500)
do
  ./target/debug/rustico -p 12 -d debug
done
