docker run -p 6667:6667 \
    -p 6697:6697 \
    -v ./inspircd.config:/inspircd/conf/ \
    inspircd/inspircd-docker
