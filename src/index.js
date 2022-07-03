const cronJob = require('cron').CronJob;
const axios = require('axios');

require('dotenv').config();

const domainId = process.env.DOMAIN_ID;
const provider = process.env.PROVIDER;
const auth = process.env.AUTH;
const recordId = process.env.RECORD_ID;

let buffer = undefined;

axios
  .get('https://www.lima-city.de/usercp/domains/31511/records.json', {
    auth: {
      username: 'api',
      password: auth,
    },
  })
  .then((res) => {
    console.log('--------------------\nInitial request...');
    res.data.records.forEach((record) => {
      if (record.id == recordId) {
        buffer = record.content;
        console.log(`${res.status}\t ${record.content}\n-------------------\n`);
      }
    });
  })
  .catch((error) => {
    console.error('No internet connection or other error...');
  });

console.log(`New job for Domain ${domainId} with provider ${provider}`);
const job = new cronJob('* * * * *', () => {
  var changed = false;
  var new_ip = undefined;
  const d = new Date();
  d.setTime(Date.now());

  const lima_city = () => {
    axios
      .get('https://api64.ipify.org?format=json')
      .then((res) => {
        if (res.data.ip != buffer) {
          new_ip = res.data.ip;

          axios
            .put(
              `https://www.lima-city.de/usercp/domains/${domainId}/records/${recordId}`,
              {
                nameserver_record: {
                  content: new_ip,
                },
              },
              {
                auth: {
                  username: 'api',
                  password: auth,
                },
              }
            )
            .then((res) => {
              if (res.status == 200) {
                changed = true;
                buffer = new_ip;
                console.log(
                  `${d.toUTCString()}\t ${changed}\t ${buffer} -> ${new_ip}`
                );
              }
            })
            .catch((error) => {
              console.error('No internet connection or other error...');
            });
        }
      })
      .catch((error) => {
        console.error('No internet connection or other error...');
      });
  };

  switch (provider) {
    case 'lima_city':
      lima_city();
      break;
    default:
      console.error('No such provider.');
  }
});
job.start();
