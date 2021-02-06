import datetime
import logging
import re
import os

def generate_message(event, template):
    with open("./src/cogs/abakus/templates/"+template, "r") as f:
        msg = f.read()
    time = datetime.datetime.strftime(event["registrationOpen"], '%Y-%m-%d %H:%M:%S')
    return (msg.format(
        eventName=event['name'],
        eventDescription=event['description'],
        signupTime=time,
        eventLocation=event['eventLocation'],
        startTime=event['eventTime'],
        url=event['url']
    ))


def get_event_properties(message, template):
    with open("./src/cogs/abakus/templates/"+template, "r") as f:
        pattern = f.read()
    messageSearch = re.search(pattern, message.content)
    if messageSearch:
        name = messageSearch.group(1)
        description = messageSearch.group(2)
        startTime = messageSearch.group(3)
        location = messageSearch.group(4)
        signupTime = messageSearch.group(5)
        url = messageSearch.group(6)
        event = {
            "name": name,
            "description": description,
            "registrationOpen": datetime.datetime.strptime(signupTime, '%Y-%m-%d %H:%M:%S'),
            "eventLocation": location,
            "eventTime": startTime,
            "url": url
        }
    else:
        event = {"registrationOpen": "None"}
    return event



async def get_dm_history(user):
    if user.dm_channel:
        pass
    else:
        await user.create_dm()

    history = await user.dm_channel.history(limit=123).flatten()
    history = [x.content for x in history]
    return history
