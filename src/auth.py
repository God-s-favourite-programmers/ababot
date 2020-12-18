import requests
import json


def getAuthCookie(usr, psw):
    url = "https://lego.abakus.no/authorization/token-auth/"
    auth = {"username": usr, "password": psw}
    r = requests.post(url, json=auth)
    token = json.loads(r.text)["token"]
    return token
