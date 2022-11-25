# Monitor metrics description
This document is a description of the monitoring metrics and all headings correspond to the dashboard on Grafana.

## axon-node
### Resource Overview 
#### Overall total 5m load & average CPU used
- type: CPU
- description: Monitor overall cpu usage
<details>
<summary>Legende details</summary>

##### CPU Cores
Number of cores for all CPUs
```
count(node_cpu_seconds_total{job=~"node_exporter", mode='system'})
```
##### Total 5m load
load5 for all CPUs
```
sum(node_load5{job=~"node_exporter"})
```
##### Overall average used%
Average utilization of all CPUs
```
avg(1 - avg(irate(node_cpu_seconds_total{job=~"node_exporter",mode="idle"}[5m])) by (instance)) * 100
```
###### Alert threshold:
Utilization rate over 60%

##### Load5 Avg
load5 Avg for all CPUs
```
sum(node_load5{job=~"node_exporter"}) / count(node_cpu_seconds_total{job=~"node_exporter", mode='system'})
```
###### Alert threshold:
Load5 Avg greater than 0.7

</details>

#### Overall total memory & average memory used
- type: Memory
- description: Monitor overall memory usage

<details>
<summary>Legende details</summary>

##### Total
Total memory
```
sum(node_memory_MemTotal_bytes{job=~"node_exporter"})
```

##### Total Used
Overall used memory
```
sum(node_memory_MemTotal_bytes{job=~"node_exporter"} - node_memory_MemAvailable_bytes{job=~"node_exporter"})
```

##### Overall Average Used%
Utilization of all memory
```
(sum(node_memory_MemTotal_bytes{job=~"node_exporter"} - node_memory_MemAvailable_bytes{job=~"node_exporter"}) / sum(node_memory_MemTotal_bytes{job=~"node_exporter"}))*100
```
###### Alert threshold:
Utilization rate over 70%

</details>


#### Overall total disk & average disk used%
- type: Disk
- description: Monitor overall disk usage

<details>
<summary>Legende details</summary>

##### Total
Total memory
```
sum(avg(node_filesystem_size_bytes{job=~"node_exporter",fstype=~"xfs|ext.*"})by(device,instance))

```

##### Total Used
Overall used disk
```
sum(avg(node_filesystem_size_bytes{job=~"node_exporter",fstype=~"xfs|ext.*"})by(device,instance)) - sum(avg(node_filesystem_free_bytes{job=~"node_exporter",fstype=~"xfs|ext.*"})by(device,instance))

```

##### Overall Average Used%
Utilization of all disk
```
(sum(avg(node_filesystem_size_bytes{job=~"node_exporter",fstype=~"xfs|ext.*"})by(device,instance)) - sum(avg(node_filesystem_free_bytes{job=~"node_exporter",fstype=~"xfs|ext.*"})by(device,instance))) *100/(sum(avg(node_filesystem_avail_bytes{job=~"node_exporter",fstype=~"xfs|ext.*"})by(device,instance))+(sum(avg(node_filesystem_size_bytes{job=~"node_exporter",fstype=~"xfs|ext.*"})by(device,instance)) - sum(avg(node_filesystem_free_bytes{job=~"node_exporter",fstype=~"xfs|ext.*"})by(device,instance))))

```
###### Alert threshold:
Utilization rate over 70%

</details>


### Resource Details
<a id="Internet-traffic-per-hour" name="Internet-traffic-per-hour"></a>					
#### Internet traffic per hour 
- type: Network
- description: Traffic statistics

<details>
<summary>Legende details</summary>

##### receive
Receive statistics
```
increase(node_network_receive_bytes_total{instance=~"$node",device=~"$device"}[60m])
```

##### transmit
transmit statistics
```
increase(node_network_transmit_bytes_total{instance=~"$node",device=~"$device"}[60m])
```
</details>

#### CPU% Basic
- type: CPU
- description: Node CPU usage

<details>
<summary>Legende details</summary>

##### System
Average sy ratio
```
avg(irate(node_cpu_seconds_total{instance=~"$node",mode="system"}[5m])) by (instance) *100
```

##### User
Average sy ratio
```
avg(irate(node_cpu_seconds_total{instance=~"$node",mode="user"}[5m])) by (instance) *100
```

##### Iowait
Average sy ratio
```
avg(irate(node_cpu_seconds_total{instance=~"$node",mode="iowait"}[5m])) by (instance) *100
```

##### Total
Average CPU usage
```
(1 - avg(irate(node_cpu_seconds_total{instance=~"$node",mode="idle"}[5m])) by (instance))*100
```

##### Average used%
Not show, for alert
```
(1 - avg(irate(node_cpu_seconds_total{mode="idle"}[5m])) by (instance)) *100
```
###### Alert threshold:
Utilization rate over 60%
</details>











#### Memory Basic
- type: Memory
- description: Node memory usage

<details>
<summary>Legende details</summary>

##### Total
Total memory
```
node_memory_MemTotal_bytes{instance=~"$node"}
```

##### Used
Used memory
```
node_memory_MemTotal_bytes{instance=~"$node"} - node_memory_MemAvailable_bytes{instance=~"$node"}

```

##### Avaliable
Available memory size
```
node_memory_MemAvailable_bytes{instance=~"$node"}

```

##### Used%
Utilization of all memory
```
(1 - (node_memory_MemAvailable_bytes{instance=~"$node"} / (node_memory_MemTotal_bytes{instance=~"$node"})))* 100
```

##### {{instance}}-Used%
Not show, for alert
```
(1 - (node_memory_MemAvailable_bytes / node_memory_MemTotal_bytes))* 100
```
###### Alert threshold:
Utilization rate over 60%
</details>





<a id="Network-bandwidth-usage-per-second-all" name="Network-bandwidth-usage-per-second-all"></a>
#### Network bandwidth usage per second all
- type: Network
- description: Network bandwidth

<details>
<summary>Legende details</summary>

##### receive
Receive statistics per second
```
irate(node_network_receive_bytes_total{instance=~'$node',device=~"$device"}[5m])*8

```

##### transmit
Transmit statistics per second
```
irate(node_network_transmit_bytes_total{instance=~'$node',device=~"$device"}[5m])*8

```
</details>


#### System Load
- type: CPU
- description: System Load

<details>
<summary>Legende details</summary>

##### 1m
Load 1
```
node_load1{instance=~"$node"}
```

##### 5m
Load 5
```
node_load5{instance=~"$node"}
```

##### 15m
Load 15
```
node_load15{instance=~"$node"}
```

##### CPU cores
Number of cores for CPU
```
sum(count(node_cpu_seconds_total{instance=~"$node", mode='system'}) by (cpu,instance)) by(instance)
```

##### Load5 Avg
load5 Avg for CPU
```
avg(node_load5{instance=~"$node"}) / count(node_cpu_seconds_total{instance=~"$node", mode='system'})
```

##### Load5 Avg-{{instance}}
Not show, for alert
```
sum(node_load5) by (instance) / count(node_cpu_seconds_total{job=~"node_exporter", mode='system'}) by (instance)
```
Alert threshold:
Load5 Avg greater than 0.7
</details>


#### Disk R/W Data
- type: Disk
- description: Disk throughput

<details>
<summary>Legende details</summary>

##### Read bytes
Read bytes
```
node_load1{instance=~"$node"}
```

##### Written bytes
Written bytes
```
node_load5{instance=~"$node"}
```
</details>

#### Disk Space Used% Basic

- type: Disk
- description: IOPS

<details>
<summary>Legende details</summary>

##### mountpoint
Disk space utilization
```
(node_filesystem_size_bytes{instance=~'$node',fstype=~"ext.*|xfs",mountpoint !~".*pod.*"}-node_filesystem_free_bytes{instance=~'$node',fstype=~"ext.*|xfs",mountpoint !~".*pod.*"}) *100/(node_filesystem_avail_bytes {instance=~'$node',fstype=~"ext.*|xfs",mountpoint !~".*pod.*"}+(node_filesystem_size_bytes{instance=~'$node',fstype=~"ext.*|xfs",mountpoint !~".*pod.*"}-node_filesystem_free_bytes{instance=~'$node',fstype=~"ext.*|xfs",mountpoint !~".*pod.*"}))

```
</details>

#### Disk IOps Completed（IOPS）
- type: Disk
- description: IOPS

<details>
<summary>Legende details</summary>

##### Reads completed
Read IOPS
```
irate(node_disk_io_time_seconds_total{instance=~"$node"}[5m])
```

##### Writes completed
Write IOPS
```
irate(node_disk_io_time_seconds_total{instance=~"(.*):9100"}[5m])
```
</details>


#### Time Spent Doing I/Os
- type: Disk
- description: I/O Utilization

<details>
<summary>Legende details</summary>

##### IO time
I/O Utilization
```
irate(node_disk_io_time_seconds_total{instance=~"$node"}[5m])
```

##### {{instance}}-%util
Not show, for alert
```
irate(node_disk_io_time_seconds_total{instance=~"(.*):9100"}[5m])
```
Alert threshold:
Utilization rate over 80%
</details>


#### Disk R/W Time(Reference: less than 100ms)(beta)
- type: Disk
- description: Average response time

<details>
<summary>Legende details</summary>

##### Read time
Read time
```
irate(node_disk_read_time_seconds_total{instance=~"$node"}[5m]) / irate(node_disk_reads_completed_total{instance=~"$node"}[5m])
```

##### Write time
Write time
```
irate(node_disk_write_time_seconds_total{instance=~"$node"}[5m]) / irate(node_disk_writes_completed_total{instance=~"$node"}[5m])
```
</details>




#### Network Sockstat
- type: Network
- description: Socket State 

<details>
<summary>Legende details</summary>

##### CurrEstab
Number of ESTABLISHED state connections
```
node_netstat_Tcp_CurrEstab{instance=~'$node'}
```

##### TCP_tw status
Number of time_wait state connections
```
node_sockstat_TCP_tw{instance=~'$node'}
```

##### Sockets_used
Total number of all protocol sockets used
```
node_sockstat_sockets_used{instance=~'$node'}
```

##### UDP_inuse
Number of UDP sockets in use
```
node_sockstat_UDP_inuse{instance=~'$node'}
```

##### TCP_alloc
Number of TCP sockets(ESTABLISHED, sk_buff)
```
node_sockstat_TCP_alloc{instance=~'$node'}
```

##### Tcp_PassiveOpens
Number of passively opened tcp connections
```
irate(node_netstat_Tcp_PassiveOpens{instance=~'$node'}[5m])
```

##### Tcp_ActiveOpens
Number of active open tcp connections
```
irate(node_netstat_Tcp_ActiveOpens{instance=~'$node'}[5m])
```

##### Tcp_InSegs
Number of tcp messages received
```
irate(node_netstat_Tcp_InSegs{instance=~'$node'}[5m])
```

##### Tcp_OutSegs
Number of tcp messages transmit
```
irate(node_netstat_Tcp_OutSegs{instance=~'$node'}[5m])
```

##### Tcp_RetransSegs
Number of tcp messages retransmitted
```
irate(node_netstat_Tcp_RetransSegs{instance=~'$node'}[5m])
```

</details>


#### Open File Descriptor(left)/Context switches(right)
- type: Disk
- description: I/O Utilization

<details>
<summary>Legende details</summary>

##### used filefd
Number of open file fd
```
node_filefd_allocated{instance=~"$node"}
```

##### switches
Context switches
```
irate(node_context_switches_total{instance=~"$node"}[5m])
```
</details>



### Actuator Health
#### Axon Status
- type: Axon
- description: Axon service status
<details>
<summary>Legende details</summary>

##### active
Number of Axon services in up status
```
count(up{job="axon_exporter"} == 1) 
```
##### down
Number of Axon services in down status
```
count(up{job="axon_exporter"} == 0) 
```

##### /
Not show, for alert
```
up{job="axon_exporter"} == 0
```
###### Alert threshold:
The value of the Metric variable up is zero

</details>	


#### Node Status
- type: Node_exporter
- description: Node_exporter service status
<details>
<summary>Legende details</summary>

##### active
Number of Node_exporter services in up status
```
count(up{job="node_exporter"} == 1) 
```
##### down
Number of Node_exporter services in down status
```
count(up{job="node_exporter"} == 0) 
```

##### down
Not show, for alert
```
up{job="node_exporter"} == 0
```
###### Alert threshold:
The value of the Metric variable up is zero

</details>	


#### Promethues Status
- type: Promethues
- description: Promethues service status
<details>
<summary>Legende details</summary>

##### active
Number of Promethues services in up status
```
count(up{job="prometheus"} == 1) 
```
##### down
Number of Promethues services in down status
```
count(up{job="prometheus"} == 0) 
```

##### down
Not show, for alert
```
up{job="prometheus"} == 0
```
###### Alert threshold:
The value of the Metric variable up is zero

</details>	



#### Jaeger Status
- type: Jaeger
- description: Jaeger service status
<details>
<summary>Legende details</summary>

##### jaeger-query-active
Number of Jaeger-query services in up status
```
count(up{instance=~"(.*):16687"} == 1)
```
##### jaeger-collector-active
Number of Jaeger-collector services in down status
```
count(up{instance=~"(.*):14269"} == 1)
```

##### jaeger-query-down
Number of Jaeger-query services in up status
```
count(up{instance=~"(.*):16687"} == 0)
```
##### jaeger-collector-down
Number of Jaeger-collector services in down status
```
count(up{instance=~"(.*):14269"} == 0)
```

##### /
Not show, for alert
```
up{instance=~"(.*):16687"} == 0
```
###### Alert threshold:
The value of the Metric variable up is zero


##### /
Not show, for alert
```
up{instance=~"(.*):14269"} == 0
```
###### Alert threshold:
The value of the Metric variable up is zero

</details>	


#### Jaeger Agent Status
- type: Jaeger
- description: Jaeger agent status
<details>
<summary>Legende details</summary>

##### active
Number of Jaeger-agent services in up status
```
count(up{job="jaeger_agent"} == 1) 
```

##### down
Number of Jaeger-agent services in down status
```
count(up{job="jaeger_agent"} == 0) 
```

##### /
Not show, for alert
```
up{job="jaeger_agent"} == 0
```
###### Alert threshold:
The value of the Metric variable up is zero

</details>	

#### Loki Status
- type: Promtail
- description: Loki service status
<details>
<summary>Legende details</summary>

##### active
Number of Loki services in up status
```
count(up{job="loki"} == 1) 
```

##### down
Number of Loki services in down status
```
count(up{job="loki"} == 0) 
```

##### /
Not show, for alert
```
up{job="loki"} == 0
```
###### Alert threshold:
The value of the Metric variable up is zero

</details>	

#### Promtail Status
- type: Promtail
- description: Promtail service status
<details>
<summary>Legende details</summary>

##### active
Number of Promtail services in up status
```
count(up{job="promtail_agent"} == 1) 
```

##### down
Number of Promtail services in down status
```
count(up{job="promtail_agent"} == 0) 
```

##### /
Not show, for alert
```
up{job="promtail_agent"} == 0
```
###### Alert threshold:
The value of the Metric variable up is zero

</details>	




## axon-benchmark
#### TPS
- description: TPS for consensus
<details>
<summary>Legende details</summary>

##### TPS
TPS for consensus
```
avg(rate(axon_consensus_committed_tx_total[5m]))
```
###### Alert threshold:
TPS is zero
</details>


#### consensus_p90
- description: Consensus time for P90
<details>
<summary>Legende details</summary>

##### time_usage(s)
Consensus time for P90
```
avg(histogram_quantile(0.90, sum(rate(axon_consensus_duration_seconds_bucket[5m])) by (le, instance)))
```

##### /
Not show, for alert
```
avg(histogram_quantile(0.90, sum(rate(axon_consensus_duration_seconds_bucket[5m])) by (le, instance))) / avg(histogram_quantile(0.90, sum(rate(axon_consensus_time_cost_seconds_bucket{type="exec"}[5m])) by (le, instance))) 
```

###### Alert threshold:
More than three rounds of consensus
</details>

#### exec_p90
- description: Consensus exec time for P90
<details>
<summary>Legende details</summary>

##### /
Consensus exec time for P90
```
avg(histogram_quantile(0.90, sum(rate(axon_consensus_time_cost_seconds_bucket{type="exec"}[5m])) by (le, instance)))
```
</details>

#### put_cf_each_block_time_usage
- description: Average time per block for rocksdb running put_cf
<details>
<summary>Legende details</summary>

##### /
Average time per block for rocksdb running put_cf
```
avg (sum by (instance) (increase(axon_storage_put_cf_seconds[5m]))) / avg(increase(axon_consensus_height[5m]))
```
</details>

#### get_cf_each_block_time_usage

- description: Average time per block for rocksdb running get_cf
<details>
<summary>Legende details</summary>

##### /
Average time per block for rocksdb running get_cf
```
avg (sum by (instance) (increase(axon_storage_get_cf_seconds[5m]))) / avg(increase(axon_consensus_height[5m]))
```
</details>


<a id="processed_tx_request" name="processed_tx_request"></a>
#### processed_tx_request
- description: received transaction request count in last 5 minutes (the unit is count/second)
<details>
<summary>Legende details</summary>

##### Total
Total number of transaction requests
```
sum(rate(axon_api_request_result_total{type="send_transaction"}[5m]))
```

##### Success Total
Total number of successful transaction requests
```
sum(rate(axon_api_request_result_total{result="success",type="send_transaction"}[5m]))
```

##### instance
processed transaction request count in last 5 minutes (the unit is count/second)
```
rate(axon_api_request_result_total{result="success", type="send_transaction"}[5m])
```
</details>


<a id="current_height" name="current_height"></a>
#### current_height
- description: Chain current height
<details>
<summary>Legende details</summary>

##### {{instance}}
Node current height

```
sort_desc(axon_consensus_height)
```
</details>

#### Liveness
- description: Liveness
<details>
<summary>Legende details</summary>

##### Liveness
Growth in node height
```
increase(axon_consensus_height{job="axon_exporter"}[1m])
```
###### Alert threshold:
Loss of Liveness

##### /
Not show, for alert
```
up{job="axon_exporter"} == 1
```
###### Alert threshold:
Loss of Liveness
</details>


<a id="synced_block" name="synced_block"></a>
#### synced_block
- description: Number of blocks synchronized by nodes
<details>
<summary>Legende details</summary>

##### {{instance}}
Number of blocks synchronized by nodes
```
axon_consensus_sync_block_total 
```
</details>


#### network_message_arrival_rate
- description: Estimate the network message arrival rate in the last five minutes
<details>
<summary>Legende details</summary>

##### /
Estimate the network message arrival rate in the last five minutes
```
(
  # broadcast_count * (instance_count - 1)
  sum(increase(axon_network_message_total{target="all", direction="sent"}[5m])) * (count(count by (instance) (axon_network_message_total)) - 1)
  # unicast_count
  + sum(increase(axon_network_message_total{target="single", direction="sent"}[5m]))
) 
/
# received_count
(sum(increase(axon_network_message_total{direction="received"}[5m])))
```
</details>


<a id="consensus_round_cost" name="Network-bandwidth-usage-per-second-all"></a>
#### consensus_round_cost
- description: Number of rounds needed to reach consensus
<details>
<summary>Legende details</summary>

##### {{instance}}
Number of rounds needed to reach consensus
```
(axon_consensus_round > 0 )
```
###### Alert threshold:
More than three rounds of consensus
</details>

<a id="mempool_cached_tx" name="mempool_cached_tx"></a>
#### mempool_cached_tx
- description: Number of transactions in the current mempool
<details>
<summary>Legende details</summary>

##### {{instance}}
Number of transactions in the current mempool
```
axon_mempool_tx_count
```
</details>

#### Connected Peers(Gauge)
- description: Number of nodes on the current connection
<details>
<summary>Legende details</summary>

##### {{instance}}
Number of nodes on the current connection
```
axon_network_connected_peers
```
</details>

#### Connected Peers(Graph)
- description: Number of nodes on the current connection
<details>
<summary>Legende details</summary>

##### Saved peers
Total number of peers
```
max(axon_network_saved_peer_count)
```

##### Connected Peers
Number of nodes on the current connection
```
axon_network_connected_peers
```
</details>


#### Consensus peers(gauge)
- description: Number of consensus nodes
<details>
<summary>Legende details</summary>

##### {{instance}}
Number of consensus nodes
```
axon_network_tagged_consensus_peers
```
</details>

#### Consensus peers(Graph)
- description: Number of consensus nodes
<details>
<summary>Legende details</summary>

##### Consensus peers
Total number of consensus peers
```
max(axon_network_tagged_consensus_peers)
```

##### {{instance}}-Connected Consensus Peers (Minus itself)
Number of consensus nodes
```
axon_network_connected_consensus_peers
```

##### /
Average utilization of all CPUs
```
(sum(axon_network_tagged_consensus_peers
) by (instance) - 1)
- sum(axon_network_connected_consensus_peers) by (instance)
```
###### Alert threshold:
Alert on loss of connection to a consensus node
</details>


#### Saved peers
- description: Number of nodes saved peers
<details>
<summary>Legende details</summary>

##### {{instance}}
Number of nodes saved peers
```
axon_network_saved_peer_count
```
</details>


<!-- #### Connected Consensus Peers (Minus itself)
- description: Number of consensus nodes on the current connection
<details>
<summary>Legende details</summary>

##### {{instance}}
Number of consensus nodes on the current connection
```
axon_network_connected_consensus_peers
```
</details> -->

#### Unidentified Connections
- description: The number of connections in the handshake, requiring verification of the chain id
<details>
<summary>Legende details</summary>

##### {{instance}}
The number of connections in the handshake, requiring verification of the chain id
```
axon_network_unidentified_connections
```
</details>

#### Connecting Peers
- description: Number of active initiations to establish connections with other machines
<details>
<summary>Legende details</summary>

##### {{instance}}
Number of active initiations to establish connections with other machines
```
axon_network_outbound_connecting_peers
```
</details>

#### Disconnected count(To other peers)
- description: Disconnected count
<details>
<summary>Legende details</summary>

##### {{instance}}
Disconnected count
```
axon_network_ip_disconnected_count
```
</details>

#### Received messages in processing
- description: Number of messages being processed
<details>
<summary>Legende details</summary>

##### {{instance}}
Number of messages being processed
```
axon_network_received_message_in_processing_guage
```
</details>

#### Received messages in processing by ip
- description: Number of messages being processed (based on IP of received messages)
<details>
<summary>Legende details</summary>

##### {{instance}}
Number of messages being processed (based on IP of received messages)
```
axon_network_received_ip_message_in_processing_guage{instance=~"$node"}
```
</details>

#### Ping (ms)_p90
- description: p90 for P2p Ping
<details>
<summary>Legende details</summary>

##### {{instance}}
p90 for P2p Ping
```
avg(histogram_quantile(0.90, sum(rate(axon_network_ping_in_ms_bucket[5m])) by (le, instance)))
```
</details>

<!-- #### Ping by ip
- description: ping response time of the current node and other nodes
<details>
<summary>Legende details</summary>

##### {{instance}}
ping response time of the current node and other nodes
```
axon_network_ip_ping_in_ms
```
</details> -->


#### Peer give up warnings
- description: Peer give up warnings
<details>
<summary>Legende details</summary>

##### Log labels
Peer give up warnings
```
{filename="/opt/axon.log"} |~ "WARN" |~ "give up"
```
</details>


## axon-network
#### Network bandwidth usage per second all
[link axon-node (Network bandwidth usage per second all)](#Network-bandwidth-usage-per-second-all)

#### Internet traffic per hour
[link axon-node (Internet traffic per hour)](#Internet-traffic-per-hour)


#### mempool_cached_tx
[link axon-benchmark (mempool_cached_tx)](#mempool_cached_tx)

#### consensus_round_cost
[link axon-benchmark (consensus_round_cost)](#consensus_round_cost)

#### current_height
[link axon-benchmark (current_height)](#current_height)

#### synced_block
[link axon-benchmark (synced_block)](#synced_block)

#### processed_tx_request
[link axon-benchmark (processed_tx_request)](#processed_tx_request)


#### height and round
- description: Height of consensus and rounds of consensus
<details>
<summary>Legende details</summary>

##### height
Height of consensus
```
axon_consensus_height{instance=~"$node"}
```

##### round
Rounds of consensus
```
(axon_consensus_round{instance=~"$node"} > 0 )
```
</details>



#### axon_network_message_size
- description: Network transmission size statistics
<details>
<summary>Legende details</summary>

##### send-total-{{instance}}
Each node send statistics
```
sum(url:axon_network_message_size:sum5m{direction="send"}) by (instance)
```

##### received-total-{{instance}}
Each node received statistics
```
sum(url:axon_network_message_size:sum5m{direction="received"}) by (instance)
```

##### send-total
Total send
```
sum(url:axon_network_message_size:sum5m{direction="send"})
```

##### received-total
Total received
```
sum(url:axon_network_message_size:sum5m{direction="received"})
```

</details>


#### network_send_by_url_size_and_count
- description: Statistics on the number and size of network send
<details>
<summary>Legende details</summary>

##### {{url}}
Size of each url send
```
sum(url:axon_network_message_size:sum5m{direction="send",instance=~"$node"}) by (url)
```

##### count_{{action}}
Count of each url send
```
sum(rate(axon_network_message_total{direction="sent"}[5m])) by (action)
```
</details>

#### network_received_by_url_size_and_count
- description: Statistics on the number and size of network send
<details>
<summary>Legende details</summary>

##### {{url}}
Size of each url received
```
sum(url:axon_network_message_size:sum5m{direction="received",instance=~"$node"}) by (url)
```

##### count_{{action}}
Count of each url received
```
sum(rate(axon_network_message_total{direction="received"}[5m])) by (action)
```
</details>