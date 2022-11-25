monitor-deploy:
	ansible-playbook -i hosts deploy_monitor.yml --tags config,down,start --ask-become-pass

monitor-clean:
	ansible-playbook -i hosts deploy_monitor.yml --tags down,clean --ask-become-pass

agent-deploy:
	ansible-playbook -i hosts deploy_monitor_agent.yml --tags config,down,start --ask-become-pass

agent-clean:
	ansible-playbook -i hosts deploy_monitor_agent.yml --tags down,clean --ask-become-pass
