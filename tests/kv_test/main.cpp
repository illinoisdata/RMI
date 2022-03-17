#include <algorithm>
#include <chrono>
#include <fcntl.h>
#include <fstream>
#include <iostream>
#include <stdio.h>
#include <stdlib.h>
#include <sys/mman.h>
#include <sys/stat.h>
#include <unistd.h>
#include <vector>
#include <math.h>

#include "flags.h"
#include "mmap_struct.h"
#include "rmi.h"

double report_t(size_t t_idx, size_t &count_milestone, size_t &last_count_milestone, long long &last_elapsed, std::chrono::time_point<std::chrono::high_resolution_clock> start_t) {
  const double freq_mul = 1.1;
  auto lookups_end_time = std::chrono::high_resolution_clock::now();
  auto lookup_time = std::chrono::duration_cast<std::chrono::nanoseconds>(
                      lookups_end_time - start_t)
                      .count();
  std::cout << "<<< " << lookup_time << " ns " << " to finish " << (t_idx + 1) << " queries." << std::endl;
  auto curr_time = std::chrono::high_resolution_clock::now();
  long long time_elapsed = std::chrono::duration_cast<std::chrono::nanoseconds>(curr_time - start_t).count();
  std::cout << "t = " << time_elapsed << " ns: " << t_idx + 1
            << " counts, tot " << (time_elapsed) / (t_idx + 1)
            << "/op, seg " << (time_elapsed - last_elapsed) / (t_idx + 1 - last_count_milestone) << "/op" << std::endl;
  last_elapsed = time_elapsed;
  last_count_milestone = count_milestone;
  count_milestone = ceil(((double) count_milestone) * freq_mul);  // next milestone to print
  return std::chrono::duration<double>(lookup_time).count();
}


int main(int argc, char* argv[]) {
  // load flags
  auto flags = parse_flags(argc, argv);

  // extract paths
  std::string data_path = get_required(flags, "data_path");
  std::string key_path = get_required(flags, "key_path");
  std::string rmi_data_path = get_required(flags, "rmi_data_path");
  std::string out_path = get_required(flags, "out_path");

  auto queries = std::vector<uint64_t>();
  auto expected_ans = std::vector<uint64_t>();
  std::ifstream query_words_in(key_path);
  std::string line;
  while (std::getline(query_words_in, line)) {
    std::istringstream input;
    input.str(line);

    std::string key;
    std::string exp;
    input >> key;
    input >> exp;

    queries.push_back(std::stoull(key));
    expected_ans.push_back(std::stoull(exp));
  }

  // start timer
  auto start_t = std::chrono::high_resolution_clock::now();

  // Load the data
  KeyArray<uint64_t> data(data_path.c_str());
  // report_t(777, start_t);

  // Load RMI
  std::cout << "RMI status: " << rmi::load(rmi_data_path.c_str()) << std::endl;
  // report_t(888, start_t);

  // for (uint64_t key_index = 0; key_index < data.size(); key_index++) {
  size_t err;
  // const auto freq_mul = 1.1;
  size_t last_count_milestone = 0;
  size_t count_milestone = 1;
  long long last_elapsed = std::chrono::duration_cast<std::chrono::nanoseconds>(start_t - start_t).count();
  auto timestamps = std::vector<double>();
  for (size_t t_idx = 0; t_idx < queries.size(); t_idx++) {
    // TODO: sample this
    uint64_t lookup = queries[t_idx];
    uint64_t answer = expected_ans[t_idx];	

    // rmi index
    size_t rmi_guess = (size_t) rmi::lookup(lookup, &err);

    // error correction
    size_t guess_left = (rmi_guess >= err) ? rmi_guess - err : 0;
    size_t guess_right = rmi_guess + err;
    size_t true_index = data.rank_within(lookup, guess_left, guess_right);
   
    if (answer != true_index) {
	std::cout << "ERROR: Incorrect RMI Index. Expected value: " << answer
		  << " RMI answer:" << true_index << std::endl; 
    }
    
    if (t_idx + 1 == count_milestone) {
	timestamps.push_back(report_t(t_idx, count_milestone, last_count_milestone, last_elapsed, start_t));    
        //auto curr_time = std::chrono::high_resolution_clock::now();
	//auto time_elapsed = std::chrono::duration_cast<std::chrono::nanoseconds>(curr_time - start_t).count();
	//std::cout << "t = " << time_elapsed << " ns: " << t_idx + 1 
	//	  << " counts, tot " << (time_elapsed) / (t_idx + 1) 
	//	  << "/op, seg " << (time_elapsed - last_elapsed) / (t_idx + 1 - last_count_milestone) << "/op" << std::endl;  
	//last_elapsed = time_elapsed;
        //last_count_milestone = count_milestone;	
	//count_milestone = ceil(((double) count_milestone) * freq_mul);  // next milestone to print       
    }

    if (t_idx % 10000 == 0) { // UNCOMMENT to debug
      // compute error, TODO: mute this?
      uint64_t diff = (rmi_guess > true_index ? rmi_guess - true_index : true_index - rmi_guess);

      // print message
      std::cout << "Search key: " << lookup
                << " RMI guess: " << rmi_guess << " +/- " << err
                << " Key at " << true_index << ": " << data[true_index]
                << " diff: " << diff << std::endl;
    }
  }

  // write result to file
  std::ofstream file_out;
  file_out.open(out_path, std::ios_base::app);
  for (const auto& timestamp : timestamps) {
    file_out << timestamp / 1000000.0 << ",";
  }
  file_out << std::endl;
  file_out.close();

  // clean up data
  // delete &data;
  
  // clean up index
  rmi::cleanup();

  return 0;
}
