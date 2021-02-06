import os
import logging

if os.path.isfile("/ababot/token.txt"):
    print("Found token.txt, attempting to use saved token")
    logging.info("Found token.txt, attempting to use saved token")
    with open("/ababot/token.txt", "r") as f:
        token = f.read()
else:
    print("token.txt file not found. Run the container with a volume, to avoid this problem in the future")
    logging.info(
        "token.txt file not found. Run the container with a volume, to avoid this problem in the future")
    token = input("Provide token manually: ")
    with open("/ababot/token.txt", "w") as f:
        f.write(token)
    print("Token written to token.txt, reuse the volume next time to avoid providing the token manually")
    logging.info("Token written to token.txt")
