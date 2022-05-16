SHELL=bash

tag=latest
registry=docker-adobe-spectrum-release.dr.corp.adobe.com
repository=lcs-png-diff-server

build-image:
	docker build \
	-t lcs-png-diff-server:temp \
	-f ./Dockerfile \
	..

	docker tag lcs-png-diff-server:temp $(registry)/$(repository):$(tag)
	docker image rm lcs-png-diff-server:temp
