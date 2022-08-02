sudo sysctl vm.drop_caches=3
sleep 10
(TIMEFORMAT=%R; time echo `cat ./random.txt | wc -l`)
echo "reload_local completed"
