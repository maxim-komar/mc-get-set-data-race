setup_hosts:
	ansible-galaxy install -r ansible/roles/requirements.yml
	ansible-playbook -i ansible/inventory.ini ansible/setup-hosts.yml
