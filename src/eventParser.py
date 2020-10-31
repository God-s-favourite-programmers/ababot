import requests
import json
import datetime
import re



def getEvent(eventId):
    url = "https://lego.abakus.no/api/v1/events/"+str(eventId)
    r = requests.get(url)
    data = json.loads(r.text)
    name = data["title"]
    description = data["description"]
    eventTime = data["startTime"]
    eventLocation = data["location"]
    rawRegistrationOpen = data["pools"][0]["activationDate"]
    regex = re.search("(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2}):(\d{2})Z", rawRegistrationOpen)
    registrationOpen = datetime.datetime(int(regex.group(1)), int(regex.group(2)), int(regex.group(3)), int(regex.group(4)), int(regex.group(5)), int(regex.group(6)))
    return {
        "name": name,
        "description": description,
        "eventTime": eventTime,
        "eventLocation": eventLocation,
        "regsitrationOpen": registrationOpen
    }


print(getEvent(2773))
